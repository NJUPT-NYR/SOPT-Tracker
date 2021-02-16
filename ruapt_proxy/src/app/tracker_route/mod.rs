mod context;
mod data;
mod announce;

use std::io::Read;

use actix_web::*;

use bendy::serde::to_bytes;
use futures::prelude::*;
use tokio::prelude::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};


#[get("/hello")]
async fn hello_world() -> impl Responder {
    std::mem::size_of::<data::AnnouncePacket>().to_string()
}

pub fn tracker_service() -> Scope {
    web::scope("/tracker")
        // thread safe?
        .data(context::Context::new())
        .service(hello_world)
        .service(announce::announce)
}
