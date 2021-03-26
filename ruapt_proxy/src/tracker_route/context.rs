use deadpool::managed;
use deadpool_redis::{redis::RedisError, Config, ConnectionWrapper};
use cuckoofilter::CuckooFilter;
use std::collections::hash_map::DefaultHasher;
use tokio::sync::RwLock;

type Pool = managed::Pool<ConnectionWrapper, RedisError>;
pub(crate) type Filter = CuckooFilter<DefaultHasher>;
pub struct Context {
    pub pool: Pool,
    pub filter: RwLock<Filter>,
    // TODO: A connection to backend
    // TODO: monitor, LOGGER are needed
}

impl Context {
    pub fn new(uri: &str) -> Self {
        let mut cfg = Config::default();
        cfg.url = Some(uri.to_string());
        let filter = RwLock::new(Filter::new());
        let pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        Context { pool, filter }
    }
}
