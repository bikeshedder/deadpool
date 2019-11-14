use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

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
    pub async fn prepare(&mut self, sql: &str) -> Result<Statement, Error> {
        let sql_string = sql.to_owned();
        match self.statement_cache.get(&sql_string) {
            Some(statement) => Ok(statement.clone()),
            None => {
                let stmt = self.client.prepare(sql).await?;
                self.statement_cache.insert(sql_string.clone(), stmt.clone());
                return Ok(stmt)
            }
        }
    }
}

impl Deref for Client {
    type Target = PgClient;
    fn deref(&self) -> &PgClient {
        &self.client
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut PgClient {
        &mut self.client
    }
}

#[tokio::main]
#[test]
async fn test_postgres() {
    use std::env;
    use crate::{Pool, Manager};
    let mut cfg = tokio_postgres::Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::new(mgr, 16);
    let mut client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
}
