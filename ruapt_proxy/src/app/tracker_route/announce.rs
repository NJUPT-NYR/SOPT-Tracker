use actix_web::*;
use io::AsyncWriteExt;
use futures::prelude::*;
use tokio::prelude::*;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use super::data::{AnnouncePacket, AnnounceRequestData};
use super::context::Context;

#[get("/announce")]
pub async fn announce(
    web::Query(mut q): web::Query<AnnounceRequestData>,
    req: HttpRequest,
    data: web::Data<Context>,
) -> impl Responder {
    // refine
    let query = req.uri().query().unwrap();
    let start = query.find("info_hash=").unwrap() + 10;
    let info_hash = &query[start..];
    let end = info_hash.find("&").unwrap_or(info_hash.len());
    let info_hash = &info_hash[..end];
    let info_hash = parse_info_hash(info_hash);
    q.info_hash = info_hash;
    let peer_ip = req.peer_addr().map(|addr| addr.ip());
    println!("{:?}", q.info_hash.as_slice());
    // q.check_validation();
    q.fix_ip(peer_ip);
    let p = AnnouncePacket::from(&q);
    //TODO: error handler
    let mut con = data.pool.get().await.unwrap();
    let (read_half, mut write_half) = con.split();
    let mut reader = FramedRead::new(read_half, LengthDelimitedCodec::new());
    println!("{:#?}", p.as_bytes());
    write_half.write_all(p.as_bytes()).await.unwrap();
    if let Ok(Some(msg)) = reader.try_next().await {
        return msg;
    }
    panic!("TODO")
}

fn decode_hex(x: u8) -> u8 {
    if x >= b'0' && x <= b'9' {
        return x - b'0';
    }
    if x >= b'a' && x <= b'f' {
        return x - b'a' + 10;
    } else {
        panic!("GG");
    }
}
fn parse_info_hash(s: &str) -> Vec<u8> {
    let it = s.as_bytes();
    let mut ret: Vec<u8> = Vec::new();
    let mut i = 0;
    while i != it.len() {
        let u: u8;
        if it[i] == b'%' {
            u = decode_hex(it[i + 1]) * 16 + decode_hex(it[i + 2]);
            i += 2;
        } else {
            u = it[i];
        }
        i += 1;
        ret.push(u);
    }
    ret
}
