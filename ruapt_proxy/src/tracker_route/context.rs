use deadpool::managed;
use deadpool_redis::{redis::RedisError, Config, ConnectionWrapper};

type Pool = managed::Pool<ConnectionWrapper, RedisError>;
pub struct Context {
    pub pool: Pool,
    // TODO: monitor, LOGGER are needed
}

impl Context {
    pub fn new(uri: &str) -> Self {
        let mut cfg = Config::default();
        cfg.url = Some(uri.to_string());
        let pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        Context { pool }
    }
}
