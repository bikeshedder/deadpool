#[cfg(feature = "managed")]
#[cfg(feature = "config")]
mod tests {

    use deadpool::managed::PoolConfig;
    use serde::Deserialize;
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

    #[derive(Debug, Deserialize)]
    struct TestConfig {
        pool: PoolConfig,
    }

    #[test]
    fn test_from_env() {
        let mut env = Env::new();
        env.set("POOL__MAX_SIZE", "42");
        env.set("POOL__TIMEOUTS__WAIT__SECS", "1");
        env.set("POOL__TIMEOUTS__WAIT__NANOS", "0");
        env.set("POOL__TIMEOUTS__CREATE__SECS", "2");
        env.set("POOL__TIMEOUTS__CREATE__NANOS", "0");
        env.set("POOL__TIMEOUTS__RECYCLE__SECS", "3");
        env.set("POOL__TIMEOUTS__RECYCLE__NANOS", "0");
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::new().separator("__"))
            .unwrap();
        let cfg: TestConfig = cfg.try_into().unwrap();
        assert_eq!(cfg.pool.max_size, 42);
        assert_eq!(cfg.pool.timeouts.wait, Some(Duration::from_secs(1)));
        assert_eq!(cfg.pool.timeouts.create, Some(Duration::from_secs(2)));
        assert_eq!(cfg.pool.timeouts.recycle, Some(Duration::from_secs(3)));
    }
}
