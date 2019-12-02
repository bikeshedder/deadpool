//! Deadpool simple async pool for PostgreSQL connections.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
//! and also provides a `statement` cache by wrapping `tokio_postgres::Client`
//! and `tokio_postgres::Transaction`.
//!
//! You should not need to use `deadpool` directly. Use the `Pool` type
//! provided by this crate instead.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::env;
//!
//! use deadpool_postgres::{Manager, Pool};
//! use tokio_postgres::{Config, NoTls};
//!
//! #[tokio::main]
//! fn main() {
//!     let mut cfg = Config::new();
//!     cfg.host("/var/run/postgresql");
//!     cfg.user(env::var("USER").unwrap().as_str());
//!     cfg.dbname("deadpool");
//!     let mgr = Manager::new(cfg tokio_postgres::NoTls);
//!     let pool = Pool::new(mgr, 16);
//!     loop {
//!         let mut client = pool.get().await.unwrap();
//!         let stmt = client.prepare("SELECT random()").await.unwrap();
//!         let rows = client.query(&stmt, &[]).await.unwrap();
//!         let value: f64 = rows[0].get(0);
//!         println!("{}", value);
//!     }
//! }
//! ```
#![warn(missing_docs)]

use std::collections::HashMap;
use std::ops::Deref;

use async_trait::async_trait;
use futures::FutureExt;
use log::{debug, warn};
use tokio::spawn;
use tokio_postgres::{
    tls::MakeTlsConnect, tls::TlsConnect, Client as PgClient, Config as PgConfig, Error, Socket,
    Statement, Transaction as PgTransaction,
};

/// A type alias for using `deadpool::Pool` with `tokio_postgres`
pub type Pool = deadpool::Pool<Client, tokio_postgres::Error>;

/// The manager for creating and recyling postgresql connections
pub struct Manager<T: MakeTlsConnect<Socket>> {
    config: PgConfig,
    tls: T,
}

impl<T: MakeTlsConnect<Socket>> Manager<T> {
    /// Create manager using `PgConfig` and a `TlsConnector`
    pub fn new(config: PgConfig, tls: T) -> Manager<T> {
        Manager {
            config: config,
            tls: tls,
        }
    }
}

#[async_trait]
impl<T> deadpool::Manager<Client, Error> for Manager<T>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    async fn create(&self) -> Result<Client, Error> {
        let (client, connection) = self.config.connect(self.tls.clone()).await?;
        let connection = connection.map(|r| {
            if let Err(e) = r {
                warn!(target: "deadpool.postgres", "Connection error: {}", e);
            }
        });
        spawn(connection);
        Ok(Client::new(client))
    }
    async fn recycle(&self, client: Client) -> Result<Client, Error> {
        if let Ok(_) = client.simple_query("").await {
            Ok(client)
        } else {
            debug!(target: "deadpool.postgres", "Recycling of DB connection failed. Reconnecting...");
            self.create().await
        }
    }
}

/// A wrapper for `tokio_postgres::Client` which includes a statement cache.
pub struct Client {
    client: PgClient,
    statement_cache: HashMap<String, Statement>,
}

impl Client {
    /// Create new wrapper instance using an existing `tokio_postgres::Client`
    pub fn new(client: PgClient) -> Client {
        Client {
            client: client,
            statement_cache: HashMap::new(),
        }
    }
    /// Creates a new prepared statement using the statement cache if possible.
    ///
    /// See [`tokio_postgres::Client::prepare`](#method.prepare-1)
    pub async fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        let query_owned = query.to_owned();
        match self.statement_cache.get(&query_owned) {
            Some(statement) => Ok(statement.clone()),
            None => {
                let stmt = self.client.prepare(query).await?;
                self.statement_cache
                    .insert(query_owned.clone(), stmt.clone());
                Ok(stmt)
            }
        }
    }
    /// Begins a new database transaction which supports the statement cache.
    ///
    /// See [`tokio_postgres::Client::transaction`](#method.transaction-1)
    pub async fn transaction<'a>(&'a mut self) -> Result<Transaction<'a>, Error> {
        Ok(Transaction {
            txn: PgClient::transaction(&mut self.client).await?,
            statement_cache: &mut self.statement_cache,
        })
    }
}

impl Deref for Client {
    type Target = PgClient;
    fn deref(&self) -> &PgClient {
        &self.client
    }
}

/// A wrapper for `tokio_postgres::Transaction` which uses the statement cache
/// from the client object it was created by.
pub struct Transaction<'a> {
    txn: PgTransaction<'a>,
    statement_cache: &'a mut HashMap<String, Statement>,
}

impl<'a> Transaction<'a> {
    /// Creates a new prepared statement using the statement cache if possible.
    ///
    /// See [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    pub async fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        let query_owned = query.to_owned();
        match self.statement_cache.get(&query_owned) {
            Some(statement) => Ok(statement.clone()),
            None => {
                let stmt = self.txn.prepare(query).await?;
                self.statement_cache
                    .insert(query_owned.clone(), stmt.clone());
                Ok(stmt)
            }
        }
    }
}

impl<'a> Deref for Transaction<'a> {
    type Target = PgTransaction<'a>;
    fn deref(&self) -> &PgTransaction<'a> {
        &self.txn
    }
}
