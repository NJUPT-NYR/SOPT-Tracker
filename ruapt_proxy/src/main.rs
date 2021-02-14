mod data;
mod tcp_pool;
use crate::data::*;
use bendy::serde::to_bytes;
use futures::prelude::*;
use std::sync::Arc;
use tcp_pool::*;
use tokio::prelude::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

async fn announce(mut con: Connection, d: &AnnounceRequestData) {
    let (read_half, write_half) = con.split();
    let mut reader = FramedRead::new(read_half, LengthDelimitedCodec::new());
    let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    let bytes = to_bytes(&d).unwrap();
    writer.send(bytes.into()).await.unwrap();
    if let Ok(Some(_msg)) = reader.try_next().await {
        return;
    }
    panic!("damn!");
}

async fn bench(pool: Arc<Pool>, c: Arc<Context>) {
    use std::time::SystemTime;
    let mut s = 0;
    let mut cnt = 0;
    // let c = Context::new();
    loop {
        let d = c.get_announce_data();
        let now = SystemTime::now();
        let con = pool.get().await.unwrap();
        announce(con, &d).await;
        let t = now.elapsed().unwrap().as_millis();
        s = (s + t) / 2;
        if cnt == 0 {
            println!("[RT] : {}ms", s);
        }
        cnt = (cnt + 1) % 1000;
    }
}

struct Context {
    users: Vec<String>,
    torrents: Vec<String>,
}

impl Context {
    pub fn new() -> Context {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        let torrents = (1..10000)
            .map(|_| thread_rng().sample_iter(&Alphanumeric).take(20).collect())
            .collect();
        let users = (1..1000)
            .map(|_| thread_rng().sample_iter(&Alphanumeric).take(20).collect())
            .collect();
        Context { users, torrents }
    }
    pub fn get_announce_data(&self) -> AnnounceRequestData {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let uid = rng.gen_range(0, self.users.len());
        let tid = rng.gen_range(0, self.torrents.len());
        AnnounceRequestData {
            info_hash: self.torrents[tid].clone(),
            peer_id: self.users[uid].clone(),
            ip: "localhost".into(),
            port: 8080,
            action: Action::Started,
            num_want: 100,
        }
    }
}

#[tokio::main]
async fn main() {
    let m = Manager::new("127.0.0.1:8081").unwrap();
    let pool = Arc::new(Pool::new(m, 1000));
    // let con = pool.get().await?;
    // bench(pool).await;
    let c = Arc::new(Context::new());
    let h: Vec<tokio::task::JoinHandle<()>> = (1..10)
        .map(|_| tokio::spawn(bench(pool.clone(), c.clone())))
        .collect();
    // todo!()
    for han in h {
        han.await;
    }
}
