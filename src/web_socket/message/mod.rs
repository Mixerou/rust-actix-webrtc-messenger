use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::error::AppError;
use crate::web_socket::connection::WebSocketConnection;
pub use crate::web_socket::message::payload::*;

mod payload;

#[derive(Debug, Default, Deserialize_repr, Serialize_repr, Eq, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    HeartBeat = 0,
    Request = 1,
    #[default]
    Response = 2,
    Error = 3,
    Authorize = 4,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Default, Deserialize, Serialize, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct WebSocketMessage {
    #[serde(rename = "i")]
    pub id: i64,
    #[serde(skip)]
    pub connection_id: i64,
    #[serde(rename = "o")]
    pub opcode: Opcode,
    #[serde(
        default = "WebSocketMessagePayload::default",
        rename = "p",
        skip_serializing_if = "WebSocketMessagePayload::is_none"
    )]
    pub payload: WebSocketMessagePayload,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct AuthorizationMessage {
    pub id: i64,
    pub connection_id: i64,
    pub token: String,
    pub address: Addr<WebSocketConnection>,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct DisconnectionMessage {
    pub connection_id: i64,
    pub registered_room_id: Option<i64>,
}
