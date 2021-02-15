mod tracker_route;
use tracker_route::*;

use actix_web::*;

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(tracker_service())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}