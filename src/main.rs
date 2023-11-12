#[macro_use]
extern crate log;

use std::env;

use actix::SystemService;
use actix_files::Files;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::get;
use actix_web::{App, HttpServer};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

use crate::utils::snowflake_generator;
use crate::web_socket::actor::WebSocket;

mod constants;
mod database;
mod error;
mod services;
mod utils;
mod web_socket;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    MessagePack,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    database::init();
    snowflake_generator::init();

    let ip = env::var("MESSENGER_IP").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("MESSENGER_PORT").unwrap_or_else(|_| "8080".into());

    WebSocket::from_registry();

    info!("Starting server on {ip} with port {port}");

    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .route("/ws", get().to(web_socket::routes::connect))
            .service(
                Files::new("", "./static")
                    .redirect_to_slash_directory()
                    .index_file("index.html")
                    .use_etag(true)
                    .use_last_modified(false)
                    .prefer_utf8(true),
            )
    })
    .bind(format!("{ip}:{port}"))?
    .run()
    .await
}
