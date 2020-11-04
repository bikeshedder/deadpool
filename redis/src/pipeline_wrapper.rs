use std::ops::{Deref, DerefMut};

use redis::{FromRedisValue, RedisResult, ToRedisArgs};
use tokio_compat_02::FutureExt;

use crate::{Cmd, ConnectionWrapper};

/// Wrapper for `redis::Cmd` which makes it compatible with the `query_async`
/// method which takes a `ConnectionLike` as argument.
///
/// This Implementation could be simplified a lot via
/// [RFC 2393](https://github.com/rust-lang/rfcs/pull/2393).
///
/// See [redis::Pipeline](https://docs.rs/redis/latest/redis/struct.Pipeline.html)
pub struct Pipeline {
    pipeline: redis::Pipeline,
}

impl Pipeline {
    /// See [redis::Pipeline::new](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.new)
    pub fn new() -> Self {
        Self {
            pipeline: redis::Pipeline::new(),
        }
    }
    /// See [redis::Pipeline::with_capacity](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.with_capacity)
    pub fn with_capacity(capacity: usize) -> Pipeline {
        Self {
            pipeline: redis::Pipeline::with_capacity(capacity),
        }
    }
    /// See [redis::Pipeline::cmd](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.cmd)
    pub fn cmd(&mut self, name: &str) -> &mut Pipeline {
        self.pipeline.cmd(name);
        self
    }
    /// See [redis::Pipeline::add_command](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.add_command)
    pub fn add_command(&mut self, cmd: Cmd) -> &mut Pipeline {
        self.pipeline.add_command(cmd.cmd);
        self
    }
    /// See [redis::Pipeline::arg](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.arg)
    pub fn arg<T: ToRedisArgs>(&mut self, arg: T) -> &mut Pipeline {
        self.pipeline.arg(arg);
        self
    }
    /// See [redis::Pipeline::ignore](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.ignore)
    pub fn ignore(&mut self) -> &mut Pipeline {
        self.pipeline.ignore();
        self
    }
    /// See [redis::Pipeline::atomic](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.atomic)
    pub fn atomic(&mut self) -> &mut Pipeline {
        self.pipeline.atomic();
        self
    }
    /// See [redis::Pipeline::query_async](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.query_async)
    pub async fn query_async<T: FromRedisValue>(
        &self,
        con: &mut ConnectionWrapper,
    ) -> RedisResult<T> {
        self.pipeline.query_async(DerefMut::deref_mut(con)).compat().await
    }
    /// See [redis::Pipeline::execute_async](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.execute_async)
    pub async fn execute_async(&self, con: &mut ConnectionWrapper) -> RedisResult<()> {
        self.query_async::<redis::Value>(con).compat().await?;
        Ok(())
    }
}

impl Deref for Pipeline {
    type Target = redis::Pipeline;
    fn deref(&self) -> &redis::Pipeline {
        &self.pipeline
    }
}

impl DerefMut for Pipeline {
    fn deref_mut(&mut self) -> &mut redis::Pipeline {
        &mut self.pipeline
    }
}

impl From<redis::Pipeline> for Pipeline {
    fn from(pipeline: redis::Pipeline) -> Self {
        Pipeline { pipeline }
    }
}

impl Into<redis::Pipeline> for Pipeline {
    fn into(self) -> redis::Pipeline {
        self.pipeline
    }
}

/// Shortcut for creating a new pipeline.
///
/// See [redis::pipe](https://docs.rs/redis/latest/redis/fn.pipe.html)
pub fn pipe() -> Pipeline {
    Pipeline::new()
}
