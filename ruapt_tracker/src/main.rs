mod config;
mod data;
mod error;
mod storage;
mod util;

use bendy::encoding::ToBencode;
use bytes::Bytes;
use data::AnnouncePacket;
use dotenv::dotenv;
use futures::prelude::*;
use std::env;
use std::sync::Arc;
use storage::redis::DB;
use storage::Storage;
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

async fn tracker_loop(socket: tokio::net::TcpStream, db: std::sync::Arc<storage::redis::DB>) {
    let (mut read_half, write_half) = socket.into_split();
    let mut writer = FramedWrite::new(write_half, LengthDelimitedCodec::new());
    let mut p = AnnouncePacket::new();
    let error_response = b"d7:failure21:internal server errore";
    loop {
        if let Err(e) = read_half.read_exact(p.as_mut_bytes()).await {
            println!("{}", e);
            return;
        } else {
            let bytes = match db.announce(&p).await {
                Ok(None) => Bytes::from_static(b""),
                Ok(Some(r)) => match r.to_bencode() {
                    Ok(v) => Bytes::from(v),
                    Err(err) => {
                        println!("{:?}", err);
                        Bytes::from_static(error_response)
                    }
                },
                Err(err) => {
                    println!("{:?}", err);
                    Bytes::from_static(error_response)
                }
            };
            if let Err(e) = writer.send(bytes).await {
                println!("{}", e);
                return;
            }
        }
    }
}

async fn compaction_loop(db: std::sync::Arc<storage::redis::DB>) {
    loop {
        // TODO: make the compaction more smooth
        if let Err(e) = db.compaction().await {
            println!("{:?}", e);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}

// jemalloc : 261% 44ms 39M
// ptmalloc : 282% 46ms 16M
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let server_addr = env::var("SERVER_ADDR")?;
    let listener = TcpListener::bind(&server_addr)
        .await
        .expect(format!("Bind to {} failed!", &server_addr).as_str());
    println!("<================Rua PT is Full Started================>");
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(tracker_loop(socket, db.clone()));
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
