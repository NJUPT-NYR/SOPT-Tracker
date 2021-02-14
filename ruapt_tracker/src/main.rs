mod data;
mod error;
mod storage;
mod util;

use bendy::serde::{from_bytes, to_bytes};
use futures::prelude::*;
use std::sync::Arc;
use storage::redis::DB;
use storage::Storage;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use crate::data::Request;

async fn tracker_loop(socket: tokio::net::TcpStream, db: std::sync::Arc<storage::redis::DB>) {
    let (read_half, write_half) = socket.into_split();
    let mut reader = FramedRead::new(read_half, LengthDelimitedCodec::new());
    let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    while let Ok(Some(msg)) = reader.try_next().await {
        let a: Request = match from_bytes(&msg) {
            Ok(a) => a,
            _ => continue,
        };
        // println!("{:?}", a);

        // Rust does support sub-typing and trait downcast may be unsound
        // so forgive me for such dull code
        match a {
            Request::Announce(req) =>
                if let Some(r) = db.announce(&req).await.unwrap() {
                    let bytes = to_bytes(&r).unwrap();
                    writer.send(bytes.into()).await.unwrap();
                },
            Request::Scrape(req) =>
                if let Some(r) = db.scrape(&req).await.unwrap() {
                    let bytes = to_bytes(&r).unwrap();
                    writer.send(bytes.into()).await.unwrap();
                }
        };
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
    println!("<================Rua PT is running================>");
    let db = Arc::new(DB::new(
        "redis://:1234567890@127.0.0.1:6379/0",
        "redis://:1234567890@127.0.0.1:6379/1",
    ));
    tokio::spawn(compaction_loop(db.clone()));
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(tracker_loop(socket, db.clone()));
    }
}
