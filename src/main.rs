#[macro_use]
extern crate log;

use std::env;

use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let ip = env::var("MESSENGER_IP").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("MESSENGER_PORT").unwrap_or_else(|_| "8080".into());

    info!("Starting server on {ip} with port {port}");

    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .route("/", web::get().to(HttpResponse::Ok))
    })
    .bind(format!("{ip}:{port}"))?
    .run()
    .await
}
