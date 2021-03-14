use crate::ProxyResult;

use super::context::Context;
use super::data::AnnounceRequestData;
use actix_web::*;

#[get("/announce")]
pub async fn announce(
    web::Query(mut q): web::Query<AnnounceRequestData>,
    req: HttpRequest,
    data: web::Data<Context>,
) -> ProxyResult {
    let peer_ip = req.peer_addr().map(|addr| addr.ip());
    q.validation()?;
    q.fix_ip(peer_ip);
    let mut cxn = data.pool.get().await.unwrap();
    let cmd = q.into_announce_cmd();
    let t: Vec<u8> = cmd.query_async(&mut cxn).await.unwrap();
    Ok(HttpResponse::Ok().body(t))
}
