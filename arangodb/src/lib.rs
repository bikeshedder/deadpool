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

use std::ops::{Deref, DerefMut};

use arangors::{uclient::ClientExt, ClientError, Connection as ArangoConnection};
use deadpool::managed;
use url::Url;

pub use arangors;

pub use self::config::{Config, ConfigError};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "arangors",
    Manager,
    managed::Object<Manager>,
    ClientError,
    ConfigError
);

/// Type alias for using [`deadpool::managed::RecycleResult`] with [`arangors`].
type RecycleResult = managed::RecycleResult<ClientError>;

/// Wrapper around [`arangors::Connection`].
///
/// This structure implements [`std::ops::Deref`] and can therefore
/// be used just like a regular [`arangors::Connection`].
#[derive(Debug)]
pub struct Connection {
    conn: Object,
}

impl Connection {
    /// Takes this [`Connection`] from its [`Pool`] permanently.
    ///
    /// This reduces the size of the [`Pool`].
    #[must_use]
    pub fn take(this: Self) -> ArangoConnection {
        Object::take(this.conn)
    }
}

impl From<Object> for Connection {
    fn from(conn: Object) -> Self {
        Self { conn }
    }
}

impl Deref for Connection {
    type Target = ArangoConnection;

    fn deref(&self) -> &ArangoConnection {
        &self.conn
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut ArangoConnection {
        &mut self.conn
    }
}

impl AsRef<ArangoConnection> for Connection {
    fn as_ref(&self) -> &ArangoConnection {
        &self.conn
    }
}

impl AsMut<ArangoConnection> for Connection {
    fn as_mut(&mut self) -> &mut ArangoConnection {
        &mut self.conn
    }
}

/// [`Manager`] for creating and recycling [`arangors`] connections.
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    url: String,
    username: String,
    password: String,
    use_jwt: bool,
}

impl Manager {
    /// Creates a new [`Manager`] with the given params.
    pub fn new(url: &str, username: &str, password: &str, use_jwt: bool) -> Self {
        Self {
            url: url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            use_jwt,
        }
    }

    /// Creates a new [`Manager`] with the given params.
    pub fn from_config(config: Config) -> Result<Self, CreatePoolError> {
        Ok(Self {
            url: config
                .url
                .ok_or(CreatePoolError::Config(ConfigError::MissingUrl))?,
            username: config
                .username
                .ok_or(CreatePoolError::Config(ConfigError::MissingUsername))?,
            password: config
                .password
                .ok_or(CreatePoolError::Config(ConfigError::MissingPassword))?,
            use_jwt: config.use_jwt,
        })
    }
}

impl managed::Manager for Manager {
    type Type = ArangoConnection;
    type Error = ClientError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = if self.use_jwt {
            ArangoConnection::establish_jwt(&self.url, &self.username, &self.password).await?
        } else {
            ArangoConnection::establish_basic_auth(&self.url, &self.username, &self.password)
                .await?
        };

        Ok(conn)
    }

    async fn recycle(&self, conn: &mut ArangoConnection, _metrics: &Metrics) -> RecycleResult {
        let url = Url::parse(&self.url).expect("Url should be valid at this point");
        conn.session()
            // I don't know if this is the correct way to do it, but TRACE should allow us to check if the connection is still open,
            // if the server answers it's open, if not, then not.
            .trace(url, String::default())
            .await
            .map(|_| ())
            .map_err(|e| managed::RecycleError::Backend(ClientError::HttpClient(e)))
    }
}
