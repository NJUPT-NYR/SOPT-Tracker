mod data;
mod error;
mod storage;
mod util;

use data::AnnouncePacket;
use env::var;
use futures::prelude::*;
use serde_bencode::{de, ser};
use std::{borrow::BorrowMut, io::Read, mem::MaybeUninit, sync::Arc};
use storage::redis::DB;
use storage::Storage;
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use dotenv::dotenv;
use std::env;

async fn tracker_loop(socket: tokio::net::TcpStream, db: std::sync::Arc<storage::redis::DB>) {
    let (mut read_half, write_half) = socket.into_split();
    let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    let mut p = AnnouncePacket::new();
    // zero copy
    // but version specify a little harder
    while let Ok(_) = read_half.read_exact(p.as_mut_bytes()).await {
        println!("{:?}", p);

        match db.announce(&p).await {
            Ok(None) => {
                //?
                let bytes = ser::to_bytes(&Option::<()>::None).unwrap();
                writer.send(bytes.into()).await.unwrap();
            }
            Ok(Some(r)) => {
                let bytes = ser::to_bytes(&r).unwrap();
                writer.send(bytes.into()).await.unwrap();
            }
            Err(err) => {
                todo!("bencode");
                writer.send("internal server error".into()).await.unwrap();
            }
        }
    }
}

async fn compaction_loop(db: std::sync::Arc<storage::redis::DB>) {
    loop {
        // TODO: make the compaction more smooth
        // TODO: here need some logs
        db.compaction().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}

// jemalloc : 261% 44ms 39M
// ptmalloc : 282% 46ms 16M
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Kill all `?`
    println!("<================Rua PT is Starting================>");

    assert_eq!(std::mem::size_of::<AnnouncePacket>(), 80);
    dotenv()?;
    let db = Arc::new(match env::var("STORAGE_ENGINE")?.as_str() {
        "redis" => {
            let torrent_uri = env::var("REDIS.TORRENT_URI")?;
            let user_uri = env::var("REDIS.USER_URI")?;
            DB::new(torrent_uri.as_str(), user_uri.as_str())
        }
        _ => {
            panic!("Unknown Storage Engine")
        }
    });
    tokio::spawn(compaction_loop(db.clone()));
    let listener = TcpListener::bind(env::var("SERVER_ADDR")?.as_str())
        .await
        .unwrap();
    println!("<================Rua PT is Full Started================>");
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(tracker_loop(socket, db.clone()));
    }
}
