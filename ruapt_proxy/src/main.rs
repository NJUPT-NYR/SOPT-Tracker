mod util;

mod config;
mod tracker_route;

mod error;
use actix_web::*;
use error::ProxyError;
use tracker_route::*;

pub type ProxyResult = Result<HttpResponse, ProxyError>;

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(tracker_service())
    })
    .bind("192.168.31.222:8080")?
    .run()
    .await
}

fn main() {
    start_server().expect("GG");
}
