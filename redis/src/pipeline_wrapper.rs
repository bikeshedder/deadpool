use futures::compat::Future01CompatExt;
use redis::{
    FromRedisValue,
    ToRedisArgs,
    RedisResult,
};

use crate::{Connection, Cmd};

/// See [redis::Pipeline](https://docs.rs/redis/latest/redis/struct.Pipeline.html)
pub struct Pipeline {
    pipeline: redis::Pipeline
}

impl Pipeline {
    /// See [redis::Pipeline::new](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.new)
    pub fn new() -> Pipeline {
        Pipeline {
            pipeline: redis::Pipeline::new()
        }
    }
    /// See [redis::Pipeline::with_capacity](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.with_capacity)
    pub fn with_capacity(capacity: usize) -> Pipeline {
        Self {
            pipeline: redis::Pipeline::with_capacity(capacity)
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
    /// See [redis::Pipeline::get_packed_pipeline](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.get_packed_pipeline)
    pub fn get_packed_pipeline(&self, atomic: bool) -> Vec<u8> {
        self.pipeline.get_packed_pipeline(atomic)
    }
    /// See [redis::Pipeline::query](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.query)
    pub async fn query<T: FromRedisValue>(
        &self,
        con: &mut Connection
    ) -> RedisResult<T> {
        let rcon = con._take_conn()?;
        let (rcon, result) = self.pipeline.clone().query_async(rcon).compat().await?;
        con._replace_conn(rcon);
        Ok(FromRedisValue::from_redis_value(&result)?)
    }
    /// See [redis::Pipeline::clear](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.clear)
    pub fn clear(&mut self) {
        self.pipeline.clear();
    }
    /// See [redis::Pipeline::execute](https://docs.rs/redis/latest/redis/struct.Pipeline.html#method.execute)
    pub async fn execute(&self, con: &mut Connection) -> RedisResult<()> {
        self.query::<redis::Value>(con).await?;
        Ok(())
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

/// See [redis::pipe](https://docs.rs/redis/0.13.0/redis/fn.pipe.html)
pub fn pipe() -> Pipeline {
    Pipeline::new()
}
