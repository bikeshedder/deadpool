use std::collections::HashMap;
use std::ops::{Deref};

use async_trait::async_trait;
use futures::FutureExt;
use log::{debug, warn};
use tokio::spawn;
use tokio_postgres::{
    Client as PgClient,
    Config as PgConfig,
    Error,
    Socket,
    Statement,
    Transaction as PgTransaction,
    tls::MakeTlsConnect,
    tls::TlsConnect,
};

pub type Pool = deadpool::Pool<Client, tokio_postgres::Error>;

pub struct Manager<T: MakeTlsConnect<Socket>> {
    config: PgConfig,
    tls: T
}

impl <T: MakeTlsConnect<Socket>> Manager<T> {
    pub fn new(config: PgConfig, tls: T) -> Manager<T> {
        Manager {
            config: config,
            tls: tls
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

pub struct Client {
    client: PgClient,
    statement_cache: HashMap<String, Statement>,
}

impl Client {
    pub fn new(client: PgClient) -> Client {
        Client {
            client: client,
            statement_cache: HashMap::new()
        }
    }
    pub async fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        let query_owned = query.to_owned();
        match self.statement_cache.get(&query_owned) {
            Some(statement) => Ok(statement.clone()),
            None => {
                let stmt = self.client.prepare(query).await?;
                self.statement_cache.insert(query_owned.clone(), stmt.clone());
                return Ok(stmt)
            }
        }
    }
    pub async fn transaction<'a>(&'a mut self) -> Result<Transaction<'a>, Error> {
        Ok(Transaction {
            txn: PgClient::transaction(&mut self.client).await?,
            statement_cache: &mut self.statement_cache
        })
    }
}

impl Deref for Client {
    type Target = PgClient;
    fn deref(&self) -> &PgClient {
        &self.client
    }
}

pub struct Transaction<'a> {
    txn: PgTransaction<'a>,
    statement_cache: &'a mut HashMap<String, Statement>,
}

impl<'a> Transaction<'a> {
    pub async fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        let query_owned = query.to_owned();
        match self.statement_cache.get(&query_owned) {
            Some(statement) => Ok(statement.clone()),
            None => {
                let stmt = self.txn.prepare(query).await?;
                self.statement_cache.insert(query_owned.clone(), stmt.clone());
                return Ok(stmt)
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
