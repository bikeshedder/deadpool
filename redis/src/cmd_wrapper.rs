use std::ops::{Deref, DerefMut};

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use tokio_compat_02::FutureExt;

use crate::ConnectionWrapper;

/// Wrapper for `redis::Cmd` which makes it compatible with the `query_async`
/// method which takes a `ConnectionLike` as argument.
///
/// This Implementation could be simplified a lot via
/// [RFC 2393](https://github.com/rust-lang/rfcs/pull/2393).
///
/// See [redis::Cmd](https://docs.rs/redis/latest/redis/struct.Cmd.html)
pub struct Cmd {
    pub(crate) cmd: redis::Cmd,
}

impl Cmd {
    /// See [redis::Cmd::new](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.new)
    pub fn new() -> Self {
        Self {
            cmd: redis::Cmd::new(),
        }
    }
    /// See [redis::Cmd::arg](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.arg)
    pub fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Cmd {
        self.cmd.arg(arg);
        self
    }
    /// See [redis::Cmd::cursor_arg](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.cursor_arg)
    pub fn cursor_arg(&mut self, cursor: u64) -> &mut Cmd {
        self.cmd.cursor_arg(cursor);
        self
    }
    /// See [redis::Cmd::query_async](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.query_async)
    pub async fn query_async<T: FromRedisValue + Send>(
        &self,
        conn: &mut ConnectionWrapper,
    ) -> RedisResult<T> {
        self.cmd
            .query_async(DerefMut::deref_mut(conn))
            .compat()
            .await
    }
    /// See [redis::Cmd::execute_async](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.execute_async)
    pub async fn execute_async(&self, con: &mut ConnectionWrapper) -> RedisResult<()> {
        self.query_async::<redis::Value>(con).compat().await?;
        Ok(())
    }
}

impl Deref for Cmd {
    type Target = redis::Cmd;
    fn deref(&self) -> &redis::Cmd {
        &self.cmd
    }
}

impl DerefMut for Cmd {
    fn deref_mut(&mut self) -> &mut redis::Cmd {
        &mut self.cmd
    }
}

impl From<redis::Cmd> for Cmd {
    fn from(cmd: redis::Cmd) -> Self {
        Cmd { cmd }
    }
}

impl Into<redis::Cmd> for Cmd {
    fn into(self) -> redis::Cmd {
        self.cmd
    }
}

/// Shortcut function to creating a command with a single argument.
///
/// The first argument of a redis command is always the name of the
/// command which needs to be a string. This is the recommended way
/// to start a command pipe.
///
/// See [redis::cmd](https://docs.rs/redis/latest/redis/fn.cmd.html)
pub fn cmd(name: &str) -> Cmd {
    let mut cmd = Cmd::new();
    cmd.arg(name);
    cmd
}
