#[cfg(feature = "config")]
mod tests {

    use deadpool::managed::PoolConfig;
    use std::collections::HashMap;
    use std::env;
    use std::time::Duration;

    struct Env {
        backup: HashMap<String, Option<String>>,
    }

    impl Env {
        pub fn new() -> Self {
            Self {
                backup: HashMap::new(),
            }
        }
        pub fn set(&mut self, name: &str, value: &str) {
            self.backup.insert(name.to_string(), env::var(name).ok());
            env::set_var(name, value);
        }
    }

    impl Drop for Env {
        fn drop(&mut self) {
            for (name, value) in self.backup.iter() {
                match value {
                    Some(value) => env::set_var(name.as_str(), value),
                    None => env::remove_var(name.as_str()),
                }
            }
        }
    }

    #[test]
    fn test_from_env() {
        let mut env = Env::new();
        env.set("POOL_MAX_SIZE", "42");
        env.set("POOL_TIMEOUTS.WAIT.SECS", "1");
        env.set("POOL_TIMEOUTS.WAIT.NANOS", "0");
        env.set("POOL_TIMEOUTS.CREATE.SECS", "2");
        env.set("POOL_TIMEOUTS.CREATE.NANOS", "0");
        env.set("POOL_TIMEOUTS.RECYCLE.SECS", "3");
        env.set("POOL_TIMEOUTS.RECYCLE.NANOS", "0");
        let cfg = PoolConfig::from_env("POOL").unwrap();
        assert_eq!(cfg.max_size, 42);
        assert_eq!(cfg.timeouts.wait, Some(Duration::from_secs(1)));
        assert_eq!(cfg.timeouts.create, Some(Duration::from_secs(2)));
        assert_eq!(cfg.timeouts.recycle, Some(Duration::from_secs(3)));
    }
}
