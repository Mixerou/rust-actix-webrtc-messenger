use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::Instant;

use actix::prelude::{Actor, Context, Handler};
use actix::{
    ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Message, Running, Supervised,
    SystemService, WrapFuture,
};
use actix_web_actors::ws::WebsocketContext;

use crate::error::{AppError, AppErrorTemplate, WebSocketCloseError};
use crate::services::room;
use crate::services::session::model::Session;
use crate::web_socket::connection::WebSocketConnection;
use crate::web_socket::message::{
    AuthorizationMessage, DisconnectionMessage, Opcode, WebSocketMessage, WebSocketMessagePayload,
};

#[derive(Debug, Default)]
pub struct WebSocket {
    connections: HashMap<i64, Addr<WebSocketConnection>>,
}

impl WebSocket {
    pub fn handle_message(
        connection: &mut WebSocketConnection,
        message: WebSocketMessage,
        context: &mut WebsocketContext<WebSocketConnection>,
    ) -> Result<(), AppError> {
        let message_id = message.id;

        if connection.session_id.is_none() {
            if message.opcode != Opcode::Authorize {
                WebSocket::close_connection(WebSocketCloseError::NotAuthenticated, context);

                return Ok(());
            }

            let WebSocketMessagePayload::Authorize { token } = message.payload else {
                return Err(AppErrorTemplate::BadRequest(None).into());
            };

            let session = match Session::find_by_token(&token) {
                Ok(session) => session,
                Err(error) => match error.http_code {
                    404 => Session::create()?,
                    _ => return Err(error),
                },
            };

            let authorization_message = AuthorizationMessage {
                id: message.id,
                connection_id: connection.id,
                token: session.token,
                address: context.address(),
            };

            connection.session_id = Some(session.id);

            WebSocket::send_message(message_id, authorization_message, connection, context);

            return Ok(());
        }

        match message.opcode {
            Opcode::HeartBeat => {
                connection.last_heartbeat_at = Instant::now();

                let response = WebSocketMessage {
                    id: message.id,
                    connection_id: message.connection_id,
                    opcode: Opcode::Response,
                    ..Default::default()
                };

                WebSocket::send_message(message_id, response, connection, context);
            }
            Opcode::Request => {
                let handle = match message.payload {
                    // Room
                    WebSocketMessagePayload::RequestGetRoomSdpOffer { .. } => {
                        room::handlers::get_sdp_offer
                    }
                    WebSocketMessagePayload::RequestPostRoomSdpAnswer { .. } => {
                        room::handlers::post_sdp_answer
                    }
                    // WebSocketMessagePayload::RequestPostRoomIceCandidate { .. } => {
                    //     room::handlers::post_ice_candidate
                    // }

                    // Other
                    _ => return Err(AppErrorTemplate::BadRequest(None).into()),
                };

                handle(message, connection, context)?;
            }
            Opcode::Response => {}
            Opcode::Authorize => {
                WebSocket::close_connection(WebSocketCloseError::AlreadyAuthenticated, context)
            }
            Opcode::Error => WebSocket::close_connection(WebSocketCloseError::Opcode, context),
        }

        Ok(())
    }

    pub fn send_message<T>(
        message_id: i64,
        message: T,
        connection: &mut WebSocketConnection,
        context: &mut WebsocketContext<WebSocketConnection>,
    ) where
        T: Message<Result = Result<(), AppError>> + Send + 'static,
        T::Result: Send,
        WebSocket: Handler<T>,
    {
        async move { WebSocket::from_registry().send(message).await? }
            .into_actor(connection)
            .map(move |result, connection, context| {
                if let Err(error) = result {
                    context.address().do_send(WebSocketMessage {
                        id: message_id,
                        connection_id: connection.id,
                        opcode: Opcode::Error,
                        payload: WebSocketMessagePayload::Response {
                            code: error.json_code,
                            message: error.get_safe_message(),
                        },
                    });
                }
            })
            .spawn(context);
    }

    fn get_connection(&self, id: &i64) -> Result<&Addr<WebSocketConnection>, AppError> {
        match self.connections.get(id) {
            Some(connection) => Ok(connection),
            None => Err(AppError::new(
                500,
                None,
                "Couldn't find WebSocket connection".to_string(),
                None,
            )),
        }
    }
}

impl Supervised for WebSocket {
    fn restarting(&mut self, _: &mut <Self as Actor>::Context) {
        warn!("WebSocket service restarting")
    }
}

impl SystemService for WebSocket {
    fn service_started(&mut self, _: &mut Context<Self>) {
        info!("WebSocket service started");
    }
}

impl Actor for WebSocket {
    type Context = Context<Self>;

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        let connections_length = self.connections.len();

        if connections_length != 0 {
            warn!("Stopping WebSocket service with active connections: {connections_length}");
        }

        Running::Stop
    }
}

impl Handler<WebSocketMessage> for WebSocket {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: WebSocketMessage, _: &mut Context<Self>) -> Self::Result {
        let connection = WebSocket::get_connection(self.borrow(), &message.connection_id)?;

        connection.do_send(message);

        Ok(())
    }
}

impl Handler<AuthorizationMessage> for WebSocket {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: AuthorizationMessage, _: &mut Context<Self>) -> Self::Result {
        self.connections
            .insert(message.connection_id, message.address);

        let connection = WebSocket::get_connection(self.borrow(), &message.connection_id)?;

        connection.do_send(WebSocketMessage {
            id: message.id,
            connection_id: message.connection_id,
            opcode: Opcode::Response,
            payload: WebSocketMessagePayload::ResponseSession {
                token: message.token,
            },
        });

        Ok(())
    }
}

impl Handler<DisconnectionMessage> for WebSocket {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: DisconnectionMessage, _: &mut Context<Self>) -> Self::Result {
        self.connections.remove(&message.connection_id);

        Ok(())
    }
}
