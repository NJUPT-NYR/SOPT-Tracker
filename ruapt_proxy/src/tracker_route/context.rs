use crate::error::ProxyError;
use deadpool::managed;
use deadpool_redis::{redis::RedisError, Config, ConnectionWrapper};
use lazy_static::lazy_static;
use rocksdb::{ColumnFamily, DB};
type Pool = managed::Pool<ConnectionWrapper, RedisError>;

lazy_static! {
    static ref ROCKSDB: DB = DB::open_cf_for_read_only(
        &Context::<'_>::rocksdb_options(),
        "./rocksdb",
        &["passkey"],
        false
    )
    .expect("Cannot open rocksdb");
}

pub struct Context<'a> {
    pub pool: Pool,
    pub rocks: &'a DB,
    pub passkey_cf: &'a ColumnFamily, // TODO: A connection to backend
                                      // TODO: monitor, LOGGER are needed
}

impl<'a> Context<'a> {
    pub fn rocksdb_options() -> rocksdb::Options {
        rocksdb::Options::default()
    }

    pub fn new(uri: &str) -> Self {
        let mut cfg = Config::default();
        cfg.url = Some(uri.to_string());
        let pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        let rocks = &ROCKSDB;
        let passkey_cf = rocks.cf_handle("passkey").expect("Cannot open clumn");
        Context {
            pool,
            rocks,
            passkey_cf,
        }
    }

    pub async fn validation(
        &self,
        data: &super::data::AnnounceRequestData,
    ) -> Result<(), crate::error::ProxyError> {
        if data.peer_id.len() != 20 {
            return Err(ProxyError::RequestError(
                "peer_id's length should be 20 bytes!",
            ));
        }
        let passkey_cf = self.rocks.cf_handle("passkey").expect("GG");
        if self.rocks.get_cf(passkey_cf, &data.passkey)?.is_none() {
            return Err(ProxyError::RequestError(
                "Passkey not found! Check your torrent please.",
            ));
        }
        Ok(())
    }
}
