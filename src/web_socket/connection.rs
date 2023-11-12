use std::time::Instant;

use actix::{Actor, ActorContext, AsyncContext, Handler, Running, StreamHandler, SystemService};
use actix_web_actors::ws;
use actix_web_actors::ws::{CloseCode, CloseReason, ProtocolError, WebsocketContext};
use rmp_serde::decode::Error as RmpSerdeDecodeError;

use crate::constants::{WEB_SOCKET_CLIENT_TIMEOUT, WEB_SOCKET_HEARTBEAT_INTERVAL};
use crate::error::{AppError, WebSocketCloseError};
use crate::utils::snowflake_generator;
use crate::web_socket::actor::WebSocket;
use crate::web_socket::message::{
    DisconnectionMessage, Opcode, WebSocketMessage, WebSocketMessagePayload,
};
use crate::Encoding;

#[derive(Debug)]
pub struct WebSocketConnection {
    pub id: i64,
    pub session_id: Option<i64>,
    pub last_heartbeat_at: Instant,
    pub encoding: Encoding,
    pub registered_room_id: Option<i64>,
}

impl WebSocketConnection {
    pub fn new(encoding: Encoding) -> Self {
        WebSocketConnection {
            id: snowflake_generator::generate(),
            session_id: None,
            last_heartbeat_at: Instant::now(),
            encoding,
            registered_room_id: None,
        }
    }

    fn heartbeat(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(WEB_SOCKET_HEARTBEAT_INTERVAL, |actor, ctx| {
            if Instant::now().duration_since(actor.last_heartbeat_at) > WEB_SOCKET_CLIENT_TIMEOUT {
                let close_reason = CloseReason {
                    code: CloseCode::Normal,
                    description: None,
                };

                ctx.close(Some(close_reason));
                ctx.stop();
            }
        });
    }

    fn send_message(
        encoding: Encoding,
        message: WebSocketMessage,
        context: &mut WebsocketContext<WebSocketConnection>,
    ) -> Result<(), AppError> {
        // Don't use the if statement to easily increase encodings.
        match encoding {
            Encoding::MessagePack => context.binary(rmp_serde::to_vec_named(&message)?),
        };

        Ok(())
    }

    fn handle_message(
        &mut self,
        message: WebSocketMessage,
        context: &mut WebsocketContext<WebSocketConnection>,
    ) {
        let message = WebSocketMessage {
            connection_id: self.id,
            ..message
        };

        let id = message.id;
        let connection_id = message.connection_id;

        if let Err(error) = WebSocket::handle_message(self, message, context) {
            WebSocketConnection::send_message(
                self.encoding,
                WebSocketMessage {
                    id,
                    connection_id,
                    opcode: Opcode::Error,
                    payload: WebSocketMessagePayload::Response {
                        code: error.json_code,
                        message: error.get_safe_message(),
                    },
                },
                context,
            )
            .unwrap();
        }
    }
}

impl Actor for WebSocketConnection {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        self.heartbeat(context);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        WebSocket::from_registry().do_send(DisconnectionMessage {
            connection_id: self.id,
            registered_room_id: self.registered_room_id,
        });

        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, message: Result<ws::Message, ProtocolError>, context: &mut Self::Context) {
        let Ok(message) = message else {
            context.stop();

            return;
        };

        match message {
            ws::Message::Text(_) => {
                WebSocket::close_connection(WebSocketCloseError::InvalidMessage, context);
            }
            ws::Message::Binary(message) => {
                let Ok(message): Result<WebSocketMessage, RmpSerdeDecodeError> =
                    rmp_serde::from_slice(&message)
                else {
                    WebSocket::close_connection(WebSocketCloseError::InvalidMessage, context);

                    return;
                };

                WebSocketConnection::handle_message(self, message, context);
            }
            ws::Message::Close(reason) => {
                context.close(reason);
                context.stop();
            }
            _ => (),
        }
    }
}

impl Handler<WebSocketMessage> for WebSocketConnection {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: WebSocketMessage, context: &mut Self::Context) -> Self::Result {
        WebSocketConnection::send_message(self.encoding, message, context)?;

        Ok(())
    }
}
