use crate::data::*;
use crate::error::*;
use crate::storage::Storage;
use crate::util::get_timestamp;
use async_trait::async_trait;
use deadpool::managed;
use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter, ErrorKind, FromRedisValue, RedisError, RedisResult, Value},
    Config, ConnectionWrapper, Pipeline, PoolError,
};
type Connection = managed::Object<ConnectionWrapper, RedisError>;
type Pool = managed::Pool<ConnectionWrapper, RedisError>;

pub struct DB {
    torrent_pool: Pool,
    user_pool: Pool,
}

impl FromRedisValue for Peer {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let gg = RedisError::from((ErrorKind::TypeError, "Cannot convert to Peer"));
        match *v {
            Value::Data(ref bytes) => Peer::from(bytes).map_err(|_| gg),
            _ => Err(gg),
        }
    }
}

impl DB {
    /// The uri format is `redis://[<username>][:<passwd>@]<hostname>[:port]`  
    /// And we will take db 1 to store torrent connect info and db 2 to store
    /// the info of users.
    pub fn new(torrent_uri: &str, user_uri: &str) -> Self {
        let mut cfg = Config::default();
        assert_ne!(torrent_uri, user_uri);
        cfg.url = Some(torrent_uri.to_string());
        let torrent_pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        cfg.url = Some(user_uri.to_string());
        let user_pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        DB {
            torrent_pool,
            user_pool,
        }
    }
}

impl DB {
    async fn get_torrent_con_with_delay(&self) -> TrackerResult<Connection> {
        loop {
            match self.torrent_pool.try_get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
                _ => return Err(TrackerError::RedisError("Pool error 1".into())),
            }
        }
    }
    async fn get_torrent_con_no_delay(&self) -> TrackerResult<Connection> {
        loop {
            match self.torrent_pool.get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => continue,
                _ => return Err(TrackerError::RedisError("Pool error 2".into())),
            }
        }
    }
    async fn get_user_con_no_delay(&self) -> TrackerResult<Connection> {
        loop {
            match self.user_pool.try_get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => continue,
                _ => return Err(TrackerError::RedisError("Pool error 3".into())),
            }
        }
    }
}

#[async_trait]
impl Storage for DB {
    async fn compaction(&self) -> TrackerResult<()> {
        let mut con1 = self.get_torrent_con_with_delay().await?;
        // fuck borrow check
        // cannot reuse con1 because the cursor take the mut borrow
        let mut con2 = self.get_torrent_con_with_delay().await?;
        let mut cursor: AsyncIter<String> = con1.scan().await?;
        let mut p = Pipeline::with_capacity(10);
        // we assume that redis is fast enough
        let now = get_timestamp();
        let mut cnt = 0;
        while let Some(key) = cursor.next_item().await {
            p.zrembyscore(&key, now - 300, now);
            cnt += 1;
            if cnt % 10 == 0 {
                p.execute_async(&mut con2).await?;
                p.clear();
            }
        }
        p.execute_async(&mut con2).await?;
        Ok(())
    }

    #[cfg(scrape = "on")]
    async fn scrape(&self, data: &ScrapeRequestData) -> TrackerResult<Option<ScrapeResponseData>> {
        let mut to_con = self.get_torrent_con_no_delay().await?;
        let t = get_timestamp();
        let mut files: HashMap<String, TorrentInfo> = HashMap::new();
        for hash in &data.info_hash {
            let total: isize = to_con.zcount(hash, t, "+inf").await?;
            let hash_ext = format!("ext_{}", &hash);
            let incomplete: isize = to_con.scard(&hash_ext).await?;
            files.insert(
                hash.clone(),
                TorrentInfo::new(total - incomplete, incomplete),
            );
        }
        Ok(Some(ScrapeResponseData { files }))
    }

    async fn announce(
        &self,
        data: &AnnounceRequestData,
    ) -> TrackerResult<Option<AnnounceResponseData>> {
        // do nothing, the compaction will remove it
        // in few minutes.
        let mut user_con = self.get_user_con_no_delay().await?;
        let mut to_con = self.get_torrent_con_no_delay().await?;
        let info_hash = format!("{}", &data.info_hash);
        let info_hash_ext = format!("ext_{}", &data.info_hash);

        if let Some(Stopped) = data.action {
            if cfg!(scrape = "on") {
                user_con.srem(&data.peer_id, &info_hash).await?;
            }
            return Ok(None);
        }
        if cfg!(scrape = "on") {
            if let Some(Completed) = data.action {
                to_con.srem(&info_hash_ext, &data.peer_id).await?;
            }
        }
        // use t_id instead info_hash to decrease memory usage
        // actually, if it is worth using t_id is unknown
        // then get the return value
        // ** A use of info_hash didn't lose so much performance **
        // ** Also better for independence from backend BY BRETHLAND **
        let mut p = Pipeline::with_capacity(4);
        if cfg!(scrape = "on") {
            p.sadd(&data.peer_id, &info_hash);
        }
        p.expire(&data.peer_id, 300);
        p.execute_async(&mut user_con).await?;
        p.clear();
        let now = get_timestamp();
        p.zadd(&info_hash, data.encode_info(), now);
        if cfg!(scrape = "on") {
            p.sadd(&info_hash_ext, &data.peer_id);
        }
        p.expire(&info_hash, 300);
        // should the extend hash be expired?
        // p.expire(&info_hash_ext, 300);
        p.execute_async(&mut to_con).await?;
        // ZRANGEBYSCORE t_id now-300 +inf LIMIT 0 num_want
        let peers: Vec<Peer> = match data.num_want {
            Some(num_want) => {
                to_con
                    .zrangebyscore_limit(&info_hash, now - 300, "+inf", 0, num_want)
                    .await?
            }
            None => to_con.zrangebyscore(&info_hash, now - 300, "+inf").await?,
        };
        Ok(Some(AnnounceResponseData { peers }))
    }
}
