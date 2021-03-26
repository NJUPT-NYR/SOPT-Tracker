use crate::ProxyResult;

use super::context::Context;
use super::data::{AnnounceRequestData,AnnounceResponseData,UpdateFilterCommand};
use actix_web::*;
use deadpool_redis::redis::Value;
use bendy::encoding::ToBencode;



#[get("/announce")]
pub async fn announce(
    web::Query(mut q): web::Query<AnnounceRequestData>,
    req: HttpRequest,
    data: web::Data<Context>,
) -> ProxyResult {
    let peer_ip = req.peer_addr().map(|addr| addr.ip());
    q.validation(&data.filter).await?;
    q.fix_ip(peer_ip);
    let mut cxn = data.pool.get().await?;
    let cmd = q.into_announce_cmd();
    let t: Vec<Value> = cmd.query_async(&mut cxn).await?;
    let response = AnnounceResponseData::from(t);
    let x = response.to_bencode()?;
    Ok(HttpResponse::Ok().body(x))
}

#[post("/updatefilter")]
pub async fn update_filter(
    web::Query(q): web::Query<UpdateFilterCommand>,
    data: web::Data<Context>
) -> ProxyResult {
    let mut filter = data.filter.write().await;
    if let Some(insert) = q.set {
        filter.add(&insert);
    }
    if let Some(delete) = q.delete {
        filter.delete(&delete);
    }
    Ok(HttpResponse::Ok().finish())
}
