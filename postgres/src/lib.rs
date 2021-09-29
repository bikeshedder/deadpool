#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

mod config;

use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock, Weak,
    },
};

use deadpool::{async_trait, managed};
use tokio::spawn;
use tokio_postgres::{
    tls::MakeTlsConnect, tls::TlsConnect, types::Type, Client as PgClient, Config as PgConfig,
    Error, IsolationLevel, Socket, Statement, Transaction as PgTransaction,
    TransactionBuilder as PgTransactionBuilder,
};

pub use deadpool::managed::reexports::*;
pub use tokio_postgres;

pub use self::config::{Config, ConfigError, ManagerConfig, RecyclingMethod};

/// Type alias for using [`deadpool::managed::Pool`] with [`tokio_postgres`].
pub type Pool = managed::Pool<Manager>;

/// Type alias for using [`deadpool::managed::Pool`] with [`tokio_postgres`].
pub type PoolBuilder = managed::PoolBuilder<Manager, Client>;

/// Type alias for using [`deadpool::managed::BuildError`] with [`tokio_postgres`].
pub type BuildError = managed::BuildError<Error>;

/// Type alias for using [`deadpool::managed::BuildError`] with [`tokio_postgres`].
pub type CreatePoolError = managed::CreatePoolError<ConfigError, Error>;

/// Type alias for using [`deadpool::managed::PoolError`] with
/// [`tokio_postgres`].
pub type PoolError = managed::PoolError<Error>;

/// Type alias for using [`deadpool::managed::Object`] with [`tokio_postgres`].
pub type Client = managed::Object<Manager>;

type RecycleResult = deadpool::managed::RecycleResult<Error>;
type RecycleError = deadpool::managed::RecycleError<Error>;

/// [`Manager`] for creating and recycling PostgreSQL connections.
///
/// [`Manager`]: managed::Manager
#[allow(missing_debug_implementations)] // due to `StatementCaches`
pub struct Manager {
    config: ManagerConfig,
    pg_config: PgConfig,
    connect: Box<dyn Connect>,
    /// [`StatementCaches`] of [`Client`]s handed out by the [`Pool`].
    pub statement_caches: StatementCaches,
}

impl Manager {
    /// Creates a new [`Manager`] using the given [`tokio_postgres::Config`] and
    /// `tls` connector.
    pub fn new<T>(pg_config: tokio_postgres::Config, tls: T) -> Self
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        Self::from_config(pg_config, tls, ManagerConfig::default())
    }

    /// Create a new [`Manager`] using the given [`tokio_postgres::Config`], and
    /// `tls` connector and [`ManagerConfig`].
    pub fn from_config<T>(pg_config: tokio_postgres::Config, tls: T, config: ManagerConfig) -> Self
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        Self {
            config,
            pg_config,
            connect: Box::new(ConnectImpl { tls }),
            statement_caches: StatementCaches::default(),
        }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = ClientWrapper;
    type Error = Error;

    async fn create(&self) -> Result<ClientWrapper, Error> {
        let client = self.connect.connect(&self.pg_config).await?;
        let client_wrapper = ClientWrapper::new(client);
        self.statement_caches
            .attach(&client_wrapper.statement_cache);
        Ok(client_wrapper)
    }

    async fn recycle(&self, client: &mut ClientWrapper) -> RecycleResult {
        if client.is_closed() {
            log::info!(target: "deadpool.postgres", "Connection could not be recycled: Connection closed");
            return Err(RecycleError::Message("Connection closed".to_string()));
        }
        match self.config.recycling_method.query() {
            Some(sql) => match client.simple_query(sql).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    log::info!(target: "deadpool.postgres", "Connection could not be recycled: {}", e);
                    Err(e.into())
                }
            },
            None => Ok(()),
        }
    }

    fn detach(&self, object: &mut ClientWrapper) {
        self.statement_caches.detach(&object.statement_cache);
    }
}

#[async_trait]
trait Connect: Sync + Send {
    async fn connect(&self, pg_config: &PgConfig) -> Result<PgClient, Error>;
}

struct ConnectImpl<T>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    tls: T,
}

#[async_trait]
impl<T> Connect for ConnectImpl<T>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    async fn connect(&self, pg_config: &PgConfig) -> Result<PgClient, Error> {
        let (client, connection) = pg_config.connect(self.tls.clone()).await?;
        drop(spawn(async move {
            if let Err(e) = connection.await {
                log::warn!(target: "deadpool.postgres", "Connection error: {}", e);
            }
        }));
        Ok(client)
    }
}

/// Structure holding a reference to all [`StatementCache`]s and providing
/// access for clearing all caches and removing single statements from them.
#[derive(Default)]
#[allow(missing_debug_implementations)] // due to `StatementCache`
pub struct StatementCaches {
    caches: Mutex<Vec<Weak<StatementCache>>>,
}

impl StatementCaches {
    fn attach(&self, cache: &Arc<StatementCache>) {
        let cache = Arc::downgrade(cache);
        self.caches.lock().unwrap().push(cache);
    }

    fn detach(&self, cache: &Arc<StatementCache>) {
        let cache = Arc::downgrade(cache);
        self.caches.lock().unwrap().retain(|sc| !sc.ptr_eq(&cache));
    }

    /// Clears [`StatementCache`] of all connections which were handed out by a
    /// [`Manager`].
    pub fn clear(&self) {
        let caches = self.caches.lock().unwrap();
        for cache in caches.iter() {
            if let Some(cache) = cache.upgrade() {
                cache.clear();
            }
        }
    }

    /// Removes statement from all caches which were handed out by a
    /// [`Manager`].
    pub fn remove(&self, query: &str, types: &[Type]) {
        let caches = self.caches.lock().unwrap();
        for cache in caches.iter() {
            if let Some(cache) = cache.upgrade() {
                drop(cache.remove(query, types));
            }
        }
    }
}

// Allows us to use owned keys in a `HashMap`, but still be able to call `get`
// with borrowed keys instead of allocating them each time.
#[derive(Debug, Eq, Hash, PartialEq)]
struct StatementCacheKey<'a> {
    query: Cow<'a, str>,
    types: Cow<'a, [Type]>,
}

/// Representation of a cache of [`Statement`]s.
///
/// [`StatementCache`] is bound to one [`Client`], and [`Statement`]s generated
/// by that [`Client`] must not be used with other [`Client`]s.
///
/// It can be used like that:
/// ```rust,ignore
/// let client = pool.get().await?;
/// let stmt = client
///     .statement_cache
///     .prepare(&client, "SELECT 1")
///     .await;
/// let rows = client.query(stmt, &[]).await?;
/// ...
/// ```
///
/// Normally, you probably want to use the [`ClientWrapper::prepare_cached()`]
/// and [`ClientWrapper::prepare_typed_cached()`] methods instead (or the
/// similar ones on [`Transaction`]).
#[allow(missing_debug_implementations)] // due to `Statement`
pub struct StatementCache {
    map: RwLock<HashMap<StatementCacheKey<'static>, Statement>>,
    size: AtomicUsize,
}

impl StatementCache {
    fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            size: AtomicUsize::new(0),
        }
    }

    /// Returns current size of this [`StatementCache`].
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Clears this [`StatementCache`].
    ///
    /// **Important:** This only clears the [`StatementCache`] of one [`Client`]
    /// instance. If you want to clear the [`StatementCache`] of all [`Client`]s
    /// you should be calling `pool.manager().statement_caches.clear()` instead.
    pub fn clear(&self) {
        let mut map = self.map.write().unwrap();
        map.clear();
        self.size.store(0, Ordering::Relaxed);
    }

    /// Removes a [`Statement`] from this [`StatementCache`].
    ///
    /// **Important:** This only removes a [`Statement`] from one [`Client`]
    /// cache. If you want to remove a [`Statement`] from all
    /// [`StatementCaches`] you should be calling
    /// `pool.manager().statement_caches.remove()` instead.
    pub fn remove(&self, query: &str, types: &[Type]) -> Option<Statement> {
        let key = StatementCacheKey {
            query: Cow::Owned(query.to_owned()),
            types: Cow::Owned(types.to_owned()),
        };
        let mut map = self.map.write().unwrap();
        let removed = map.remove(&key);
        if removed.is_some() {
            let _ = self.size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Returns a [`Statement`] from this [`StatementCache`].
    fn get(&self, query: &str, types: &[Type]) -> Option<Statement> {
        let key = StatementCacheKey {
            query: Cow::Borrowed(query),
            types: Cow::Borrowed(types),
        };
        self.map.read().unwrap().get(&key).map(ToOwned::to_owned)
    }

    /// Inserts a [`Statement`] into this [`StatementCache`].
    fn insert(&self, query: &str, types: &[Type], stmt: Statement) {
        let key = StatementCacheKey {
            query: Cow::Owned(query.to_owned()),
            types: Cow::Owned(types.to_owned()),
        };
        let mut map = self.map.write().unwrap();
        if map.insert(key, stmt).is_none() {
            let _ = self.size.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Creates a new prepared [`Statement`] using this [`StatementCache`], if
    /// possible.
    ///
    /// See [`tokio_postgres::Client::prepare()`].
    pub async fn prepare(&self, client: &PgClient, query: &str) -> Result<Statement, Error> {
        self.prepare_typed(client, query, &[]).await
    }

    /// Creates a new prepared [`Statement`] with specifying its [`Type`]s
    /// explicitly using this [`StatementCache`], if possible.
    ///
    /// See [`tokio_postgres::Client::prepare_typed()`].
    pub async fn prepare_typed(
        &self,
        client: &PgClient,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        match self.get(query, types) {
            Some(statement) => Ok(statement),
            None => {
                let stmt = client.prepare_typed(query, types).await?;
                self.insert(query, types, stmt.clone());
                Ok(stmt)
            }
        }
    }
}

/// Wrapper around [`tokio_postgres::Client`] with a [`StatementCache`].
#[allow(missing_debug_implementations)] // due to `StatementCache`
pub struct ClientWrapper {
    /// Original [`PgClient`].
    client: PgClient,

    /// [`StatementCache`] of this client.
    pub statement_cache: Arc<StatementCache>,
}

impl ClientWrapper {
    /// Create a new [`ClientWrapper`] instance using the given
    /// [`tokio_postgres::Client`].
    #[must_use]
    pub fn new(client: PgClient) -> Self {
        Self {
            client,
            statement_cache: Arc::new(StatementCache::new()),
        }
    }

    /// Like [`tokio_postgres::Transaction::prepare()`], but uses an existing
    /// [`Statement`] from the [`StatementCache`] if possible.
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        self.statement_cache.prepare(&self.client, query).await
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed()`], but uses an
    /// existing [`Statement`] from the [`StatementCache`] if possible.
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        self.statement_cache
            .prepare_typed(&self.client, query, types)
            .await
    }

    /// Like [`tokio_postgres::Client::transaction()`], but returns a wrapped
    /// [`Transaction`] with a [`StatementCache`].
    #[allow(unused_lifetimes)] // false positive
    pub async fn transaction(&mut self) -> Result<Transaction<'_>, Error> {
        Ok(Transaction {
            txn: PgClient::transaction(&mut self.client).await?,
            statement_cache: self.statement_cache.clone(),
        })
    }

    /// Like [`tokio_postgres::Client::build_transaction()`], but creates a
    /// wrapped [`Transaction`] with a [`StatementCache`].
    pub fn build_transaction(&mut self) -> TransactionBuilder<'_> {
        TransactionBuilder {
            builder: self.client.build_transaction(),
            statement_cache: self.statement_cache.clone(),
        }
    }
}

impl Deref for ClientWrapper {
    type Target = PgClient;

    fn deref(&self) -> &PgClient {
        &self.client
    }
}

impl DerefMut for ClientWrapper {
    fn deref_mut(&mut self) -> &mut PgClient {
        &mut self.client
    }
}

/// Wrapper around [`tokio_postgres::Transaction`] with a [`StatementCache`]
/// from the [`Client`] object it was created by.
#[allow(missing_debug_implementations)] // due to `StatementCache`
pub struct Transaction<'a> {
    /// Original [`PgTransaction`].
    txn: PgTransaction<'a>,

    /// [`StatementCache`] of this [`Transaction`].
    statement_cache: Arc<StatementCache>,
}

impl<'a> Transaction<'a> {
    /// Like [`tokio_postgres::Transaction::prepare()`], but uses an existing
    /// [`Statement`] from the [`StatementCache`] if possible.
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        self.statement_cache.prepare(self.client(), query).await
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed()`], but uses an
    /// existing [`Statement`] from the [`StatementCache`] if possible.
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        self.statement_cache
            .prepare_typed(self.client(), query, types)
            .await
    }

    /// Like [`tokio_postgres::Transaction::commit()`].
    pub async fn commit(self) -> Result<(), Error> {
        self.txn.commit().await
    }

    /// Like [`tokio_postgres::Transaction::rollback()`].
    pub async fn rollback(self) -> Result<(), Error> {
        self.txn.rollback().await
    }

    /// Like [`tokio_postgres::Transaction::transaction()`], but returns a
    /// wrapped [`Transaction`] with a [`StatementCache`].
    #[allow(unused_lifetimes)] // false positive
    pub async fn transaction(&mut self) -> Result<Transaction<'_>, Error> {
        Ok(Transaction {
            txn: PgTransaction::transaction(&mut self.txn).await?,
            statement_cache: self.statement_cache.clone(),
        })
    }

    /// Like [`tokio_postgres::Transaction::savepoint()`], but returns a wrapped
    /// [`Transaction`] with a [`StatementCache`].
    #[allow(unused_lifetimes)] // false positive
    pub async fn savepoint<I>(&mut self, name: I) -> Result<Transaction<'_>, Error>
    where
        I: Into<String>,
    {
        Ok(Transaction {
            txn: PgTransaction::savepoint(&mut self.txn, name).await?,
            statement_cache: self.statement_cache.clone(),
        })
    }
}

impl<'a> Deref for Transaction<'a> {
    type Target = PgTransaction<'a>;

    fn deref(&self) -> &PgTransaction<'a> {
        &self.txn
    }
}

impl<'a> DerefMut for Transaction<'a> {
    fn deref_mut(&mut self) -> &mut PgTransaction<'a> {
        &mut self.txn
    }
}

/// Wrapper around [`tokio_postgres::TransactionBuilder`] with a
/// [`StatementCache`] from the [`Client`] object it was created by.
#[allow(missing_debug_implementations)] // due to `StatementCache`
#[must_use = "builder does nothing itself, use `.start()` to use it"]
pub struct TransactionBuilder<'a> {
    /// Original [`PgTransactionBuilder`].
    builder: PgTransactionBuilder<'a>,

    /// [`StatementCache`] of this [`TransactionBuilder`].
    statement_cache: Arc<StatementCache>,
}

impl<'a> TransactionBuilder<'a> {
    /// Sets the isolation level of the transaction.
    ///
    /// Like [`tokio_postgres::TransactionBuilder::isolation_level()`].
    pub fn isolation_level(self, isolation_level: IsolationLevel) -> Self {
        Self {
            builder: self.builder.isolation_level(isolation_level),
            statement_cache: self.statement_cache,
        }
    }

    /// Sets the access mode of the transaction.
    ///
    /// Like [`tokio_postgres::TransactionBuilder::read_only()`].
    pub fn read_only(self, read_only: bool) -> Self {
        Self {
            builder: self.builder.read_only(read_only),
            statement_cache: self.statement_cache,
        }
    }

    /// Sets the deferrability of the transaction.
    ///
    /// If the transaction is also serializable and read only, creation
    /// of the transaction may block, but when it completes the transaction
    /// is able to run with less overhead and a guarantee that it will not
    /// be aborted due to serialization failure.
    ///
    /// Like [`tokio_postgres::TransactionBuilder::deferrable()`].
    pub fn deferrable(self, deferrable: bool) -> Self {
        Self {
            builder: self.builder.deferrable(deferrable),
            statement_cache: self.statement_cache,
        }
    }

    /// Begins the [`Transaction`].
    ///
    /// The transaction will roll back by default - use the commit method
    /// to commit it.
    ///
    /// Like [`tokio_postgres::TransactionBuilder::start()`].
    pub async fn start(self) -> Result<Transaction<'a>, Error> {
        Ok(Transaction {
            txn: self.builder.start().await?,
            statement_cache: self.statement_cache,
        })
    }
}

impl<'a> Deref for TransactionBuilder<'a> {
    type Target = PgTransactionBuilder<'a>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<'a> DerefMut for TransactionBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}
