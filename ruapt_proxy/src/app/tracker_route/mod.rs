mod context;
mod data;
mod announce;

use actix_web::*;

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
