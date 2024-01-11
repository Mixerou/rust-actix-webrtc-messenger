use std::fmt;

use actix::{ActorContext, MailboxError as ActixMailboxError};
use actix_web_actors::ws::{CloseCode, CloseReason, WebsocketContext};
use persy::PersyError;
use rmp_serde::decode::Error as RmpSerdeDecodeError;
use rmp_serde::encode::Error as RmpSerdeEncodeError;
use serde::Deserialize;
use structsy::StructsyError;
use webrtc::data::Error as WebRtcDataError;
use webrtc::Error as WebRtcError;

use crate::web_socket::actor::WebSocket;
use crate::web_socket::connection::WebSocketConnection;

#[derive(Debug)]
pub enum AppErrorKind {
    ActixMailboxError(ActixMailboxError),
    RmpSerdeDecodeError(RmpSerdeDecodeError),
    RmpSerdeEncodeError(RmpSerdeEncodeError),
    StructsyError(StructsyError),
    WebRtcDataError(WebRtcDataError),
    WebRtcError(WebRtcError),
    Other(Option<String>),
}

impl Default for AppErrorKind {
    fn default() -> Self {
        AppErrorKind::Other(None)
    }
}

#[derive(Debug, Deserialize)]
pub struct AppError {
    #[serde(skip)]
    pub http_code: u16,
    #[serde(rename(deserialize = "code"))]
    pub json_code: u32,
    pub message: String,
    #[serde(skip)]
    pub kind: AppErrorKind,
}

impl AppError {
    pub fn new(
        http_code: u16,
        json_code: Option<u32>,
        message: String,
        error: Option<AppErrorKind>,
    ) -> AppError {
        AppError {
            http_code,
            json_code: json_code.unwrap_or(http_code as u32),
            message,
            kind: error.unwrap_or(AppErrorKind::Other(None)),
        }
    }

    pub fn get_safe_message(&self) -> String {
        match self.http_code < 500 {
            true => self.message.to_owned(),
            false => {
                error!("{}", self.message);
                "Internal server error".to_string()
            }
        }
    }
}

impl From<ActixMailboxError> for AppError {
    fn from(error: ActixMailboxError) -> Self {
        AppError::new(
            500,
            None,
            format!("Actix mailbox error: {error}"),
            Some(AppErrorKind::ActixMailboxError(error)),
        )
    }
}

impl From<RmpSerdeDecodeError> for AppError {
    fn from(error: RmpSerdeDecodeError) -> Self {
        AppError::new(
            500,
            None,
            format!("RMP serde decode error: {error}"),
            Some(AppErrorKind::RmpSerdeDecodeError(error)),
        )
    }
}

impl From<RmpSerdeEncodeError> for AppError {
    fn from(error: RmpSerdeEncodeError) -> Self {
        AppError::new(
            500,
            None,
            format!("RMP serde encode error: {error}"),
            Some(AppErrorKind::RmpSerdeEncodeError(error)),
        )
    }
}

impl From<StructsyError> for AppError {
    fn from(error: StructsyError) -> Self {
        if let StructsyError::PersyError(persy_error) = &error {
            match persy_error {
                PersyError::RecordNotFound(_) => {
                    return AppErrorTemplate::NotFound(Some(AppErrorKind::StructsyError(error)))
                        .into();
                }
                PersyError::IndexDuplicateKey(_, _) => {
                    return AppErrorTemplate::Conflict(Some(AppErrorKind::StructsyError(error)))
                        .into();
                }
                _ => {}
            };
        }

        AppError::new(
            500,
            None,
            format!("Structsy error: {error}"),
            Some(AppErrorKind::StructsyError(error)),
        )
    }
}

impl From<WebRtcDataError> for AppError {
    fn from(error: WebRtcDataError) -> Self {
        AppError::new(
            500,
            None,
            format!("WebRTC Data error: {error}"),
            Some(AppErrorKind::WebRtcDataError(error)),
        )
    }
}

impl From<WebRtcError> for AppError {
    fn from(error: WebRtcError) -> Self {
        AppError::new(
            500,
            None,
            format!("WebRTC error: {error}"),
            Some(AppErrorKind::WebRtcError(error)),
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.message.as_str())
    }
}

macro_rules! app_error_template {
    ($(($http_code:expr, $json_code:expr, $name:ident, $message:expr);)+) => {
        pub enum AppErrorTemplate {
            $( $name(Option<AppErrorKind>), )+
        }

        impl From<AppErrorTemplate> for AppError {
            fn from(template: AppErrorTemplate) -> AppError {
                match template {
                $(
                    AppErrorTemplate::$name(error) => {
                        AppError::new($http_code, $json_code, $message.to_string(), error)
                    }
                )+
                }
            }
        }
    }
}

app_error_template! {
    // Default HTTP errors
    (400, None, BadRequest, "Bad request");
    (401, None, Unauthorized, "Unauthorized");
    (404, None, NotFound, "Not found");
    (409, None, Conflict, "Method not allowed");
    (500, None, InternalServerError, "Internal server error");

    // Minimum / Maximum number of ... reached
    (400, Some(3001), RoomNameTooShort, "Room name is too short");
    (400, Some(3002), RoomNameTooLong, "Room name is too long");
    (400, Some(3003), UsernameTooShort, "Username is too short");
    (400, Some(3004), UsernameTooLong, "Username is too long");
    (400, Some(3005), MessageContentTooShort, "Message content is too short");
    (400, Some(3006), MessageContentTooLong, "Message content is too long");

    // Invalid body or something else
    (400, Some(4001), UsernameTaken, "The username is taken");
    (400, Some(4002), WebRtcOfferNotRequested, "WebRTC offer wasn't requested");
}

macro_rules! websocket_close_error {
    ( $( ($code:expr, $name:ident, $description:expr); )+ ) => {
        #[allow(dead_code)]
        #[repr(u16)]
        pub enum WebSocketCloseError {
            $( $name = $code, )+
        }

        impl WebSocket {
            pub fn close_connection(
                error: WebSocketCloseError,
                context: &mut WebsocketContext<WebSocketConnection>,
            ) {
                match error {
                    $(
                        WebSocketCloseError::$name => {
                            let close_reason = CloseReason {
                                code: CloseCode::Other($code),
                                description: Some($description.to_string()),
                            };

                            context.close(Some(close_reason));
                            context.stop();
                        }
                    )+
                }
            }
        }
    }
}

websocket_close_error! {
    (4000, Unknown, "Unknown error");
    (4001, Opcode, "Opcode not allowed");
    (4002, InvalidMessage, "Invalid message");
    (4003, NotAuthenticated, "Not authenticated");
    (4004, AuthenticationFailed, "Authentication failed");
    (4005, AlreadyAuthenticated, "Already authenticated");
}
