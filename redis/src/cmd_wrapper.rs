use futures::compat::Future01CompatExt;
use redis::{aio::Connection as RedisConnection, FromRedisValue, RedisResult, ToRedisArgs};

use crate::ConnectionWrapper;

/// See [redis::Cmd](https://docs.rs/redis/latest/redis/struct.Cmd.html)
pub struct Cmd {
    pub(crate) cmd: redis::Cmd,
}

impl Cmd {
    /// See [redis::Cmd::new](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.new)
    pub fn new() -> Cmd {
        Cmd {
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
    /// See [redis::Cmd::get_packed_command](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.get_packed_command)
    pub fn get_packed_command(&self) -> Vec<u8> {
        self.cmd.get_packed_command()
    }
    /// See [redis::Cmd::in_scan_mode](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.in_scan_mode)
    pub fn in_scan_mode(&self) -> bool {
        self.cmd.in_scan_mode()
    }
    /// See [redis::Cmd::query](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.query)
    pub async fn query<T: FromRedisValue + Send>(
        &self,
        conn: &mut ConnectionWrapper,
    ) -> RedisResult<T> {
        let rconn = conn._take_conn()?;
        let (rconn, result) = self.cmd.query_async(rconn).compat().await?;
        conn._replace_conn(rconn);
        Ok(FromRedisValue::from_redis_value(&result)?)
    }
    /// See [redis::Cmd::execute](https://docs.rs/redis/latest/redis/struct.Cmd.html#method.execute)
    pub async fn execute(&self, conn: &mut ConnectionWrapper) -> RedisResult<()> {
        let rconn = conn._take_conn()?;
        let (rconn, _) = self
            .cmd
            .query_async::<RedisConnection, ()>(rconn)
            .compat()
            .await?;
        conn._replace_conn(rconn);
        Ok(())
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

/// See [redis::cmd](https://docs.rs/redis/0.13.0/redis/fn.cmd.html)
pub fn cmd(name: &str) -> Cmd {
    let mut cmd = Cmd::new();
    cmd.arg(name);
    cmd
}
