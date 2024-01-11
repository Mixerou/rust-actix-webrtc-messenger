use std::collections::HashMap;
use std::time::Instant;

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message,
    Running, Supervised, SystemService, WrapFuture,
};

use crate::error::{AppError, AppErrorTemplate};
use crate::services::message;
use crate::services::user::model::User;
use crate::web_rtc::connection::WebRtcConnection;
use crate::web_rtc::message::{
    DisconnectionMessage, MessageUpdateMessage, Opcode, RegistrationMessage, UserUpdateMessage,
};
use crate::web_rtc::message::{WebRtcMessage, WebRtcMessagePayload};

#[derive(Debug, Default)]
pub struct WebRtc {
    connections: HashMap<i64, Addr<WebRtcConnection>>,
}

impl WebRtc {
    pub fn handle_message(
        connection: &mut WebRtcConnection,
        message: WebRtcMessage,
        context: &mut Context<WebRtcConnection>,
    ) -> Result<(), AppError> {
        match message.opcode {
            Opcode::HeartBeat => {
                connection.last_heartbeat_at = Instant::now();

                let response = WebRtcMessage {
                    id: message.id,
                    connection_id: message.connection_id,
                    opcode: Opcode::Response,
                    ..Default::default()
                };

                WebRtc::send_message(message.id, response, connection, context);
            }
            Opcode::Request => {
                let handle = match message.payload {
                    WebRtcMessagePayload::RequestPostMessage { .. } => {
                        message::handlers::post_message
                    }
                    // Other
                    _ => return Err(AppErrorTemplate::BadRequest(None).into()),
                };

                handle(message, connection, context)?;
            }
            Opcode::Response => {}
            Opcode::Error => {}
            Opcode::Dispatch => {}
            Opcode::Hello => {}
        }
        // Opcode::Error => WebRtc::close_connection(WebRtcCloseError::Opcode, context),

        Ok(())
    }

    pub fn send_message<T>(
        message_id: i64,
        message: T,
        connection: &mut WebRtcConnection,
        context: &mut Context<WebRtcConnection>,
    ) where
        T: Message<Result = Result<(), AppError>> + Send + 'static,
        T::Result: Send,
        WebRtc: Handler<T>,
    {
        async move { WebRtc::from_registry().send(message).await? }
            .into_actor(connection)
            .map(move |result, connection, context| {
                if let Err(error) = result {
                    context.address().do_send(WebRtcMessage {
                        id: message_id,
                        connection_id: connection.id,
                        opcode: Opcode::Error,
                        payload: WebRtcMessagePayload::Response {
                            code: error.json_code,
                            message: error.get_safe_message(),
                        },
                    });
                }
            })
            .spawn(context);
    }

    fn get_connection(&self, id: &i64) -> Result<&Addr<WebRtcConnection>, AppError> {
        match self.connections.get(id) {
            Some(connection) => Ok(connection),
            None => Err(AppError::new(
                500,
                None,
                "Couldn't find WebRtc connection".to_string(),
                None,
            )),
        }
    }
}

impl Supervised for WebRtc {
    fn restarting(&mut self, _: &mut <Self as Actor>::Context) {
        warn!("WebRtc service restarting")
    }
}

impl SystemService for WebRtc {
    fn service_started(&mut self, _: &mut Context<Self>) {
        info!("WebRtc service started");
    }
}

impl Actor for WebRtc {
    type Context = Context<Self>;

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        let connections_length = self.connections.len();

        if connections_length != 0 {
            warn!("Stopping WebRtc service with active connections: {connections_length}");
        }

        Running::Stop
    }
}

impl Handler<WebRtcMessage> for WebRtc {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: WebRtcMessage, _: &mut Context<Self>) -> Self::Result {
        let connection = self.get_connection(&message.connection_id)?;

        connection.do_send(message);

        Ok(())
    }
}

impl Handler<RegistrationMessage> for WebRtc {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: RegistrationMessage, _: &mut Context<Self>) -> Self::Result {
        self.connections
            .insert(message.connection_id, message.address);

        Ok(())
    }
}

impl Handler<UserUpdateMessage> for WebRtc {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: UserUpdateMessage, _: &mut Context<Self>) -> Self::Result {
        let users = User::find_all_by_room_id(&message.room_id)?;

        for connection_id in users.iter().flat_map(|user| &user.active_connection_ids) {
            let Ok(connection) = self.get_connection(connection_id) else {
                continue;
            };

            let message = WebRtcMessage {
                id: -1,
                connection_id: *connection_id,
                opcode: Opcode::Dispatch,
                payload: WebRtcMessagePayload::DispatchUserUpdate {
                    user: message.user.clone().into(),
                },
            };

            connection.do_send(message);
        }

        Ok(())
    }
}

impl Handler<MessageUpdateMessage> for WebRtc {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: MessageUpdateMessage, _: &mut Context<Self>) -> Self::Result {
        let users = User::find_all_by_room_id(&message.room_id)?;

        for connection_id in users.iter().flat_map(|user| &user.active_connection_ids) {
            let Ok(connection) = self.get_connection(connection_id) else {
                continue;
            };

            let message = WebRtcMessage {
                id: -1,
                connection_id: *connection_id,
                opcode: Opcode::Dispatch,
                payload: WebRtcMessagePayload::DispatchMessageUpdate {
                    message: message.message.clone().into(),
                },
            };

            connection.do_send(message);
        }

        Ok(())
    }
}

impl Handler<DisconnectionMessage> for WebRtc {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: DisconnectionMessage, _: &mut Context<Self>) -> Self::Result {
        self.connections.remove(&message.connection_id);

        Ok(())
    }
}
