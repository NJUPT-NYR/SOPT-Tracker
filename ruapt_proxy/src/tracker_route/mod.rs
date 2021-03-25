mod context;
mod data;
mod announce;

use actix_web::*;
use crate::config::CONFIG;

pub fn tracker_service() -> Scope {
    web::scope("/tracker")
        .data(context::Context::new(&CONFIG.redis_uri))
        .service(announce::announce)
}
