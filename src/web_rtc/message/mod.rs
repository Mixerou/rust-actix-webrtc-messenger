use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::error::AppError;
use crate::services::message::model;
use crate::services::user::model::User;
use crate::web_rtc::connection::WebRtcConnection;
pub use crate::web_rtc::message::payload::*;

mod payload;

#[derive(Debug, Default, Deserialize_repr, Serialize_repr, Eq, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    HeartBeat = 0,
    Request = 1,
    #[default]
    Response = 2,
    Error = 3,
    Dispatch = 4,
    Hello = 5,
}

#[derive(Debug, Default, Deserialize, Serialize, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct WebRtcMessage {
    #[serde(rename = "i")]
    pub id: i64,
    #[serde(skip)]
    pub connection_id: i64,
    #[serde(rename = "o")]
    pub opcode: Opcode,
    #[serde(
        default = "WebRtcMessagePayload::default",
        rename = "p",
        skip_serializing_if = "WebRtcMessagePayload::is_none"
    )]
    pub payload: WebRtcMessagePayload,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct RegistrationMessage {
    pub connection_id: i64,
    pub address: Addr<WebRtcConnection>,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct UserUpdateMessage {
    pub user: User,
    pub room_id: i64,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct MessageUpdateMessage {
    pub message: model::Message,
    pub room_id: i64,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct DisconnectionMessage {
    pub connection_id: i64,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct RtcAnswerConnectionMessage {
    pub sdp: String,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct RtcCandidateConnectionMessage {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
    pub username_fragment: Option<String>,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct HelloConnectionMessage;

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct SendToServiceHandlerConnectionMessage {
    pub message: WebRtcMessage,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), AppError>")]
pub struct CloseConnectionMessage;
