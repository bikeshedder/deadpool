//! # Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)
//!
//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
//! and also provides a `statement` cache by wrapping `tokio_postgres::Client`
//! and `tokio_postgres::Transaction`.
//!
//! ## Features
//!
//! | Feature | Description | Extra dependencies | Default |
//! | ------- | ----------- | ------------------ | ------- |
//! | `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
//! | `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
//! | `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |
//!
//! **Important:** `async-std` support is currently limited to the
//! `async-std` specific timeout function. You still need to enable
//! the `tokio1` feature of `async-std` in order to use this crate
//! with `async-std`.
//!
//! ## Example
//!
//! ```rust,ignore
//! use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod};
//! use tokio_postgres::NoTls;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut cfg = Config::new();
//!     cfg.dbname = Some("deadpool".to_string());
//!     cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
//!     let pool = cfg.create_pool(NoTls).unwrap();
//!     for i in 1..10 {
//!         let mut client = pool.get().await.unwrap();
//!         let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
//!         let rows = client.query(&stmt, &[&i]).await.unwrap();
//!         let value: i32 = rows[0].get(0);
//!         assert_eq!(value, i + 1);
//!     }
//! }
//! ```
//!
//! ## Example with `config` and `dotenv` crate
//!
//! ```env
//! # .env
//! PG__DBNAME=deadpool
//! ```
//!
//! ```rust
//! use deadpool_postgres::{Manager, Pool};
//! use dotenv::dotenv;
//! use serde::Deserialize;
//! use tokio_postgres::NoTls;
//!
//! #[derive(Debug, Deserialize)]
//! struct Config {
//!     pg: deadpool_postgres::Config
//! }
//!
//! impl Config {
//!     pub fn from_env() -> Result<Self, ::config_crate::ConfigError> {
//!         let mut cfg = ::config_crate::Config::new();
//!         cfg.set_default("pg.dbname", "deadpool");
//!         cfg.merge(::config_crate::Environment::new().separator("__"))?;
//!         cfg.try_into()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     dotenv().ok();
//!     let mut cfg = Config::from_env().unwrap();
//!     let pool = cfg.pg.create_pool(NoTls).unwrap();
//!     for i in 1..10 {
//!         let mut client = pool.get().await.unwrap();
//!         let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
//!         let rows = client.query(&stmt, &[&i]).await.unwrap();
//!         let value: i32 = rows[0].get(0);
//!         assert_eq!(value, i + 1);
//!     }
//! }
//! ```
//!
//! **Note:** The code above uses the crate name `config_crate` because of the
//! `config` feature and both features and dependencies share the same namespace.
//! In your own code you will probably want to use `::config::ConfigError` and
//! `::config::Config` instead.
//!
//! ## Example using an existing `tokio_postgres::Config` object
//!
//! ```rust,ignore
//! use std::env;
//! use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
//! use tokio_postgres::NoTls;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut pg_config = tokio_postgres::Config::new();
//!     pg_config.host_path("/run/postgresql");
//!     pg_config.host_path("/tmp");
//!     pg_config.user(env::var("USER").unwrap().as_str());
//!     pg_config.dbname("deadpool");
//!     let mgr_config = ManagerConfig {
//!         recycling_method: RecyclingMethod::Fast
//!     };
//!     let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
//!     let pool = Pool::new(mgr, 16);
//!     for i in 1..10 {
//!         let mut client = pool.get().await.unwrap();
//!         let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
//!         let rows = client.query(&stmt, &[&i]).await.unwrap();
//!         let value: i32 = rows[0].get(0);
//!         assert_eq!(value, i + 1);
//!     }
//! }
//! ```
//!
//! ## FAQ
//!
//! - **The database is unreachable. Why does the pool creation not fail?**
//!
//!   Deadpool has [identical startup and runtime behaviour](https://crates.io/crates/deadpool/#reasons-for-yet-another-connection-pool)
//!   and therefore the pool creation will never fail.
//!
//!   If you want your application to crash on startup if no database
//!   connection can be established just call `pool.get().await` right after
//!   creating the pool.
//!
//! - **Why are connections retrieved from the pool sometimes unuseable?**
//!
//!   In `deadpool-postgres 0.5.5` a new recycling method was implemented which
//!   is the default since `0.8`. With that recycling method the manager no
//!   longer performs a test query prior returning the connection but relies
//!   solely on `tokio_postgres::Client::is_closed` instead. Under some rare
//!   circumstances (e.g. unreliable networks) this can lead to `tokio_postgres`
//!   not noticing a disconnect and reporting the connection as useable.
//!
//!   The old and slightly slower recycling method can be enabled by setting
//!   `ManagerConfig::recycling_method` to `RecyclingMethod::Verified` or when
//!   using the `config` crate by setting `PG__MANAGER__RECYCLING_METHOD=Verified`.
//!
//! - **How can I enable features of the `tokio-postgres` crate?**
//!
//!   Make sure that you depend on the same version of `tokio-postgres` as
//!   `deadpool-postgres` does and enable the needed features in your own
//!   `Crate.toml` file:
//!
//!   ```toml
//!   [dependencies]
//!   deadpool-postgres = { version = "0.7" }
//!   tokio-postgres = { version = "0.7", features = ["with-uuid-0_8"] }
//!   ```
//!
//!   **Important:** The version numbers of `deadpool-postgres` and
//!   `tokio-postgres` do not necessarily match. It is just a coincidence
//!   that both crates have the same MAJOR and MINOR version number at the
//!   time of this writing.
//!
//! - **How can I clear the statement cache?**
//!
//!   You can call `pool.manager().statement_cache.clear()` to clear all
//!   statement caches or `pool.manager().statement_cache.remove()` to remove
//!   a single statement from all caches.
//!
//!   **Important:** The `ClientWrapper` also provides a `statement_cache`
//!   field which has `clear()` and `remove()` methods which only affect
//!   a single client.
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
#![warn(missing_docs, unreachable_pub)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};

use async_trait::async_trait;
use futures::FutureExt;
use log::{info, warn};
use tokio::spawn;
use tokio_postgres::{
    tls::MakeTlsConnect, tls::TlsConnect, types::Type, Client as PgClient, Config as PgConfig,
    Error, IsolationLevel, Socket, Statement, Transaction as PgTransaction,
    TransactionBuilder as PgTransactionBuilder,
};

pub mod config;
pub use crate::config::{Config, ManagerConfig, RecyclingMethod};

/// Re-export deadpool::managed::PoolConfig
pub use deadpool::managed::PoolConfig;
/// Re-export deadpool::Runtime;
pub use deadpool::Runtime;

/// A type alias for using `deadpool::Pool` with `tokio_postgres`
pub type Pool = deadpool::managed::Pool<Manager>;

/// A type alias for using `deadpool::PoolError` with `tokio_postgres`
pub type PoolError = deadpool::managed::PoolError<tokio_postgres::Error>;

/// A type alias for using `deadpool::Object` with `tokio_postgres`
pub type Client = deadpool::managed::Object<Manager>;

type RecycleResult = deadpool::managed::RecycleResult<Error>;
type RecycleError = deadpool::managed::RecycleError<Error>;

/// Re-export tokio_postgres crate
pub use tokio_postgres;

/// The manager for creating and recyling postgresql connections
pub struct Manager {
    config: ManagerConfig,
    pg_config: PgConfig,
    connect: Box<dyn Connect>,
    /// This field provides access to the statement caches of clients
    /// handed out by the pool.
    pub statement_caches: StatementCaches,
}

impl Manager {
    /// Create manager using a `tokio_postgres::Config` and a `TlsConnector`.
    pub fn new<T>(pg_config: tokio_postgres::Config, tls: T) -> Manager
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        Self::from_config(pg_config, tls, ManagerConfig::default())
    }
    /// Create manager using a `tokio_postgres::Config` and a `TlsConnector`
    /// and `deadpool_postgres::ManagerConfig`.
    pub fn from_config<T>(
        pg_config: tokio_postgres::Config,
        tls: T,
        config: ManagerConfig,
    ) -> Manager
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        Manager {
            config,
            pg_config,
            connect: Box::new(ConnectImpl { tls }),
            statement_caches: StatementCaches::default(),
        }
    }
}

#[async_trait]
impl deadpool::managed::Manager for Manager {
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
            info!(target: "deadpool.postgres", "Connection could not be recycled: Connection closed");
            return Err(RecycleError::Message("Connection closed".to_string()));
        }
        match self.config.recycling_method.query() {
            Some(sql) => match client.simple_query(sql).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    info!(target: "deadpool.postgres", "Connection could not be recycled: {}", e);
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

#[async_trait::async_trait]
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

#[async_trait::async_trait]
impl<T> Connect for ConnectImpl<T>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    async fn connect(&self, pg_config: &PgConfig) -> Result<PgClient, Error> {
        let (client, connection) = pg_config.connect(self.tls.clone()).await?;
        let connection = connection.map(|r| {
            if let Err(e) = r {
                warn!(target: "deadpool.postgres", "Connection error: {}", e);
            }
        });
        spawn(connection);
        Ok(client)
    }
}

/// This structure holds a reference to all statement caches and provides
/// access for clearing all caches and removing single statements from them.
#[derive(Default)]
pub struct StatementCaches {
    caches: Mutex<Vec<Weak<StatementCache>>>,
}

impl StatementCaches {
    fn attach(&self, cache: &Arc<StatementCache>) {
        let cache = Arc::downgrade(&cache);
        self.caches.lock().unwrap().push(cache)
    }
    fn detach(&self, cache: &Arc<StatementCache>) {
        let cache = Arc::downgrade(&cache);
        self.caches.lock().unwrap().retain(|sc| !sc.ptr_eq(&cache));
    }
    /// Clear statement cache of all connections which were handed out by
    /// the manager.
    pub fn clear(&self) {
        let caches = self.caches.lock().unwrap();
        for cache in caches.iter() {
            if let Some(cache) = cache.upgrade() {
                cache.clear();
            }
        }
    }
    /// Remove statement from all caches which were handed out by the
    /// manager.
    pub fn remove(&self, query: &str, types: &[Type]) {
        let caches = self.caches.lock().unwrap();
        for cache in caches.iter() {
            if let Some(cache) = cache.upgrade() {
                cache.remove(query, types);
            }
        }
    }
}

// Allows us to use owned keys in the `HashMap`, but still be able
// to call `get` with borrowed keys instead of allocating them each time.
#[derive(Hash, Eq, PartialEq)]
struct StatementCacheKey<'a> {
    query: Cow<'a, str>,
    types: Cow<'a, [Type]>,
}

/// This structure provides access to the statement cache. The statement
/// cache is bound to one client and statements generated by one client
/// must not be used with other clients.
///
/// It can be used like that:
///
/// ```rust,ignore
/// let client = pool.get().await?;
/// let stmt = client
///     .statement_cache
///     .prepare(&client, "SELECT 1")
///     .await;
/// let rows = client.query(stmt, &[]).await?;
/// ...
///
/// Normally you probably want to use the `prepare_cached`
/// and `prepare_typed_cached` methods from the `ClientWrapper`
/// and `Transaction` structs instead.
/// ```
pub struct StatementCache {
    map: RwLock<HashMap<StatementCacheKey<'static>, Statement>>,
    size: AtomicUsize,
}

impl StatementCache {
    fn new() -> StatementCache {
        StatementCache {
            map: RwLock::new(HashMap::new()),
            size: AtomicUsize::new(0),
        }
    }
    /// Retrieve current size of the cache
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    /// Clear cache
    ///
    /// **Important:** This only clears the statement cache of one client
    /// instance. If you want to clear the statement cache of all clients
    /// you should be calling `pool.manager().statement_caches.clear()`
    /// instead.
    pub fn clear(&self) {
        let mut map = self.map.write().unwrap();
        map.clear();
        self.size.store(0, Ordering::Relaxed);
    }
    /// Remove statement from cache
    ///
    /// **Important:** This only removes the statement from one client
    /// cache. If you want to remove a statement from all statement caches
    /// you should be calling `pool.manager().statement_caches.remove()`
    /// instead.
    pub fn remove(&self, query: &str, types: &[Type]) -> Option<Statement> {
        let key = StatementCacheKey {
            query: Cow::Owned(query.to_owned()),
            types: Cow::Owned(types.to_owned()),
        };
        let mut map = self.map.write().unwrap();
        let removed = map.remove(&key);
        if removed.is_some() {
            self.size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }
    /// Get statement from cache
    fn get(&self, query: &str, types: &[Type]) -> Option<Statement> {
        let key = StatementCacheKey {
            query: Cow::Borrowed(query),
            types: Cow::Borrowed(types),
        };
        self.map
            .read()
            .unwrap()
            .get(&key)
            .map(|stmt| stmt.to_owned())
    }
    /// Insert statement into cache
    fn insert(&self, query: &str, types: &[Type], stmt: Statement) {
        let key = StatementCacheKey {
            query: Cow::Owned(query.to_owned()),
            types: Cow::Owned(types.to_owned()),
        };
        let mut map = self.map.write().unwrap();
        if map.insert(key, stmt).is_none() {
            self.size.fetch_add(1, Ordering::Relaxed);
        }
    }
    /// Creates a new prepared statement using the statement cache if possible.
    ///
    /// See [`tokio_postgres::Client::prepare`](#method.prepare-1)
    pub async fn prepare(&self, client: &PgClient, query: &str) -> Result<Statement, Error> {
        self.prepare_typed(client, query, &[]).await
    }
    /// Creates a new prepared statement using the statement cache if possible.
    ///
    /// See [`tokio_postgres::Client::prepare_typed`](#method.prepare_typed-1)
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

/// A wrapper for `tokio_postgres::Client` which includes a statement cache.
pub struct ClientWrapper {
    client: PgClient,
    /// The statement cache
    pub statement_cache: Arc<StatementCache>,
}

impl ClientWrapper {
    /// Create new wrapper instance using an existing `tokio_postgres::Client`
    pub fn new(client: PgClient) -> Self {
        Self {
            client,
            statement_cache: Arc::new(StatementCache::new()),
        }
    }
    /// Like [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    /// but uses an existing statement from the cache if possible.
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        self.statement_cache.prepare(&self.client, query).await
    }
    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        self.statement_cache
            .prepare_typed(&self.client, query, types)
            .await
    }
    /// Like [`tokio_postgres::Client::transaction`](#method.transaction-1)
    /// but returns a `deadpool-postgres` wrapped transaction with a statement cache.
    pub async fn transaction(&mut self) -> Result<Transaction<'_>, Error> {
        Ok(Transaction {
            txn: PgClient::transaction(&mut self.client).await?,
            statement_cache: self.statement_cache.clone(),
        })
    }
    /// Like [`tokio_postgres::Client::transaction_builder`](#method.transaction_builder-1)
    /// but creates a `deadpool-postgres` wrapped transaction with a statement cache.
    pub fn build_transaction(&mut self) -> TransactionBuilder {
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

/// A wrapper for `tokio_postgres::Transaction` which uses the statement cache
/// from the client object it was created by.
pub struct Transaction<'a> {
    txn: PgTransaction<'a>,
    /// The statement cache
    statement_cache: Arc<StatementCache>,
}

impl<'a> Transaction<'a> {
    /// Like [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    /// but uses an existing statement from the cache if possible.
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        self.statement_cache.prepare(&self.client(), query).await
    }
    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, Error> {
        self.statement_cache
            .prepare_typed(&self.client(), query, types)
            .await
    }
    /// Like [`tokio_postgres::Transaction::commit`](#method.commit-1)
    pub async fn commit(self) -> Result<(), Error> {
        self.txn.commit().await
    }
    /// Like [`tokio_postgres::Transaction::rollback`](#method.rollback-1)
    pub async fn rollback(self) -> Result<(), Error> {
        self.txn.rollback().await
    }
    /// Like [`tokio_postgres::Transaction::transaction`](#method.transaction-1)
    /// but returns a `deadpool-postgres` wrapped transaction with
    /// statement cache support.
    pub async fn transaction(&mut self) -> Result<Transaction<'_>, Error> {
        Ok(Transaction {
            txn: PgTransaction::transaction(&mut self.txn).await?,
            statement_cache: self.statement_cache.clone(),
        })
    }
    /// Like [`tokio_postgres::Transaction::savepoint`](#method.savepoint-1)
    /// but returns a `deadpool-postgres` wrapped transaction with
    /// statement cache support.
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

/// A wrapper for `tokio_postgres::TransactionBuilder` which uses the
/// statement cache from the client object it was created by.
pub struct TransactionBuilder<'a> {
    builder: PgTransactionBuilder<'a>,
    statement_cache: Arc<StatementCache>,
}

impl<'a> TransactionBuilder<'a> {
    /// Sets the isolation level of the transaction.
    ///
    /// Like `tokio_postgres::TransactionBuilder::isolation_level`
    pub fn isolation_level(self, isolation_level: IsolationLevel) -> Self {
        Self {
            builder: self.builder.isolation_level(isolation_level),
            statement_cache: self.statement_cache,
        }
    }
    /// Sets the access mode of the transaction.
    ///
    /// Like `tokio_postgres::TransactionBuilder::read_only`
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
    /// Like `tokio_postgres::TransactionBuilder::deferrable`
    pub fn deferrable(self, deferrable: bool) -> Self {
        Self {
            builder: self.builder.deferrable(deferrable),
            statement_cache: self.statement_cache,
        }
    }
    /// Begins the transaction.
    ///
    /// The transaction will roll back by default - use the commit method
    /// to commit it.
    ///
    /// Like `tokio_postgres::TransactionBuilder::start`
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
