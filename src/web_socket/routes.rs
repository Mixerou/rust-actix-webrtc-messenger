use actix_web::web::{Payload, Query};
use actix_web::{Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};

use crate::web_socket::connection::WebSocketConnection;
use crate::Encoding;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectQueryParams {
    encoding: Encoding,
}

pub async fn connect(
    request: HttpRequest,
    stream: Payload,
    params: Query<ConnectQueryParams>,
) -> Result<HttpResponse, Error> {
    ws::start(WebSocketConnection::new(params.encoding), &request, stream)
}
