mod context;
mod data;
mod announce;

use actix_web::*;

pub fn tracker_service() -> Scope {
    web::scope("/tracker")
        // thread safe?
        .data(context::Context::new("redis://127.0.0.1:6379/"))
        .service(announce::announce)
}
