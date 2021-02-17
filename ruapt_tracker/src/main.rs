mod data;
mod error;
mod storage;
mod util;

use crate::data::AnnounceRequestData;
use data::AnnouncePacket;
use futures::prelude::*;
use serde_bencode::{de, ser};
use std::{borrow::BorrowMut, io::Read, mem::MaybeUninit, sync::Arc};
use storage::redis::DB;
use storage::Storage;
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

async fn tracker_loop(socket: tokio::net::TcpStream, db: std::sync::Arc<storage::redis::DB>) {
    let (mut read_half, write_half) = socket.into_split();
    // let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    let mut buffer: [u8; 80] = [0; 80];
    read_half.read_exact(&mut buffer).await;
    let p = AnnouncePacket::init_from_buffer(&buffer);
    println!("{:?}", p);
    // while let Ok(Some(msg)) = reader.try_next().await {
    //     let a: AnnounceRequestData = match de::from_bytes(&msg) {
    //         Ok(a) => a,
    //         _ => continue,
    //     };

    //     if let Some(r) = db.announce(&a).await.unwrap() {
    //         let bytes = ser::to_bytes(&r).unwrap();
    //         writer.send(bytes.into()).await.unwrap();
    //     }

    //     // Rust does not support sub-typing and trait downcast may be unsound
    //     // so forgive me for such dull code
    //     // TODO: use backend for dispatch
    //     // match a {
    //     //     Request::Announce(req) =>
    //     //         if let Some(r) = db.announce(&req).await.unwrap() {
    //     //             let bytes = to_bytes(&r).unwrap();
    //     //             writer.send(bytes.into()).await.unwrap();
    //     //         },
    //     //     Request::Scrape(req) =>
    //     //         if let Some(r) = db.scrape(&req).await.unwrap() {
    //     //             let bytes = to_bytes(&r).unwrap();
    //     //             writer.send(bytes.into()).await.unwrap();
    //     //         }
    //     // };
    // }
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
    // TODO: use config file
    let db = Arc::new(DB::new(
        "redis://:1234567890@127.0.0.1:6379/0",
        "redis://:1234567890@127.0.0.1:6379/1",
    ));
    tokio::spawn(compaction_loop(db.clone()));
    let listener = TcpListener::bind("127.0.0.1:8082").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(tracker_loop(socket, db.clone()));
    }
}
