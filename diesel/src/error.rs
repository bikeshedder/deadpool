use std::fmt;

/// The error used when managing connections with `deadpool`.
#[derive(Debug)]
pub enum Error {
    /// An error occurred establishing the connection
    ConnectionError(diesel::ConnectionError),

    /// An error occurred pinging the database
    QueryError(diesel::result::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConnectionError(ref e) => e.fmt(f),
            Error::QueryError(ref e) => e.fmt(f),
        }
    }
}

impl ::std::error::Error for Error {}
