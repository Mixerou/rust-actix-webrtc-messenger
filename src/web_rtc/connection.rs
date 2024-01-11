use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, Running, SystemService, WrapFuture,
};
use actix_web::rt::time;
use educe::Educe;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::DataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::constants::{
    WEB_RTC_CLIENT_TIMEOUT, WEB_RTC_DATA_CHANNEL_BUFFER_SIZE, WEB_RTC_HEARTBEAT_INTERVAL,
};
use crate::error::{AppError, AppErrorTemplate};
use crate::services::message::model::Message;
use crate::services::user::model::User;
use crate::web_rtc::actor::WebRtc;
use crate::web_rtc::message::{
    CloseConnectionMessage, DisconnectionMessage, HelloConnectionMessage, Opcode,
    RegistrationMessage, RtcAnswerConnectionMessage, SendToServiceHandlerConnectionMessage,
    WebRtcMessage, WebRtcMessagePayload,
};
use crate::web_socket::connection::WebSocketConnection;
use crate::web_socket::message::{WebSocketMessage, WebSocketMessagePayload};
use crate::{web_socket, Encoding};

#[derive(Educe)]
#[educe(Debug)]
pub struct WebRtcConnection {
    pub id: i64,
    pub last_heartbeat_at: Instant,
    pub encoding: Encoding,
    pub registered_room_id: i64,
    pub registered_user_id: i64,
    pub web_socket_connection: Arc<Addr<WebSocketConnection>>,
    pub peer_connection: Arc<RTCPeerConnection>,
    #[educe(Debug(ignore))]
    pub data_channel_for_reader: Arc<Mutex<Option<Arc<DataChannel>>>>,
    pub data_channel_for_writer: Arc<Mutex<Option<Arc<DataChannel>>>>,
    pub is_closing_connection: bool,
}

impl WebRtcConnection {
    pub async fn try_new(
        id: i64,
        encoding: Encoding,
        registered_room_id: i64,
        registered_user_id: i64,
        web_socket_connection: Addr<WebSocketConnection>,
    ) -> Result<Self, AppError> {
        let web_socket_connection = Arc::new(web_socket_connection);
        let mut media_engine = MediaEngine::default();
        let mut setting_engine = SettingEngine::default();
        let registry = Registry::new();

        let Ok(registry) = register_default_interceptors(registry, &mut media_engine) else {
            return Err(AppErrorTemplate::InternalServerError(None).into());
        };

        setting_engine.detach_data_channels();

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .with_setting_engine(setting_engine)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".into()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let Ok(peer_connection) = api.new_peer_connection(config).await else {
            return Err(AppErrorTemplate::InternalServerError(None).into());
        };
        let peer_connection = Arc::new(peer_connection);

        Ok(Self {
            id,
            last_heartbeat_at: Instant::now(),
            encoding,
            registered_room_id,
            registered_user_id,
            web_socket_connection,
            peer_connection,
            data_channel_for_reader: Arc::new(Mutex::new(None)),
            data_channel_for_writer: Arc::new(Mutex::new(None)),
            is_closing_connection: false,
        })
    }

    fn init_peer_connection(&mut self, context: &mut Context<Self>) {
        let peer_connection = self.peer_connection.clone();
        let data_channel_for_reader = self.data_channel_for_reader.clone();
        let data_channel_for_writer = self.data_channel_for_writer.clone();
        let address = context.address();

        let init = async move {
            let Ok(data_channel) = peer_connection.create_data_channel("", None).await else {
                return Err(AppErrorTemplate::InternalServerError(None).into());
            };
            let data_channel_cloned = Arc::clone(&data_channel);

            data_channel.on_open(Box::new(move || {
                let data_channel_cloned = Arc::clone(&data_channel_cloned);
                Box::pin(async move {
                    // TODO: Better error handling
                    let Ok(data_channel_detached) = data_channel_cloned.detach().await else {
                        return;
                    };

                    if let Ok(mut data_channel) = data_channel_for_reader.lock() {
                        *data_channel = Some(data_channel_detached.clone());
                    }

                    if let Ok(mut data_channel) = data_channel_for_writer.lock() {
                        *data_channel = Some(data_channel_detached);
                    }

                    address.do_send(HelloConnectionMessage);
                })
            }));

            Ok(())
        };

        init.into_actor(self)
            .map(move |result: Result<(), AppError>, connection, _| {
                if result.is_err() {
                    connection
                        .web_socket_connection
                        .do_send(web_socket::message::CloseConnectionMessage);
                }
            })
            .spawn(context);
    }

    async fn create_offer(
        peer_connection: Arc<RTCPeerConnection>,
    ) -> Result<RTCSessionDescription, AppError> {
        let offer = peer_connection.create_offer(None).await?;
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        peer_connection.set_local_description(offer).await?;

        let _ = gather_complete.recv().await;
        let sdp = match peer_connection.local_description().await {
            Some(sdp) => sdp,
            None => return Err(AppErrorTemplate::InternalServerError(None).into()),
        };

        Ok(sdp)
    }

    fn heartbeat(&self, context: &mut Context<Self>) {
        context.run_interval(WEB_RTC_HEARTBEAT_INTERVAL, |actor, context| {
            if Instant::now().duration_since(actor.last_heartbeat_at) > WEB_RTC_CLIENT_TIMEOUT {
                context.address().do_send(CloseConnectionMessage);
            }
        });
    }

    pub fn send_message(
        encoding: Encoding,
        message: WebRtcMessage,
        connection: &mut Self,
        context: &mut Context<Self>,
    ) -> Result<(), AppError> {
        // Don't use the if statement to easily increase encodings.
        match encoding {
            Encoding::MessagePack => {
                let data_channel = connection.data_channel_for_writer.clone();

                async move {
                    let Ok(data_channel) = data_channel.lock() else {
                        return Ok(());
                    };
                    let Some(ref data_channel) = *data_channel else {
                        return Ok(());
                    };

                    data_channel
                        .write_data_channel(&rmp_serde::to_vec_named(&message)?.into(), false)
                        .await?;
                    Ok(())
                }
                .into_actor(connection)
                .map(|result: Result<(), AppError>, _, context| {
                    if result.is_err() {
                        context.address().do_send(CloseConnectionMessage)
                    }
                })
                .spawn(context);
            }
        };

        Ok(())
    }

    async fn receive_message(
        data_channel: Arc<DataChannel>,
        connection_id: i64,
        address: Addr<Self>,
    ) -> Result<(), AppError> {
        let mut buffer = vec![0u8; WEB_RTC_DATA_CHANNEL_BUFFER_SIZE];

        let Ok(length) = data_channel.read(&mut buffer).await else {
            return Err(AppErrorTemplate::BadRequest(None).into());
        };

        if length == 0 {
            return Err(AppErrorTemplate::BadRequest(None).into());
        }

        let Ok(message): Result<WebRtcMessage, _> = rmp_serde::from_slice(&buffer[..length]) else {
            return Err(AppErrorTemplate::BadRequest(None).into());
        };

        address.do_send(SendToServiceHandlerConnectionMessage {
            message: WebRtcMessage {
                connection_id,
                ..message
            },
        });

        Ok(())
    }
}

impl Actor for WebRtcConnection {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        self.init_peer_connection(context);

        let connection_id = self.id;
        let peer_connection = Arc::clone(&self.peer_connection);

        async move { Self::create_offer(peer_connection).await }
            .into_actor(self)
            .map(
                move |result: Result<RTCSessionDescription, AppError>, connection, _| match result {
                    Ok(offer) => connection.web_socket_connection.do_send(WebSocketMessage {
                        id: -1,
                        connection_id,
                        opcode: web_socket::message::Opcode::Response,
                        payload: WebSocketMessagePayload::ResponseRoomRtcOffer { sdp: offer.sdp },
                    }),
                    Err(_) => connection
                        .web_socket_connection
                        .do_send(web_socket::message::CloseConnectionMessage),
                },
            )
            .spawn(context);

        WebRtc::from_registry().do_send(RegistrationMessage {
            connection_id,
            address: context.address(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        WebRtc::from_registry().do_send(DisconnectionMessage {
            connection_id: self.id,
        });

        Running::Stop
    }
}

impl Handler<WebRtcMessage> for WebRtcConnection {
    type Result = Result<(), AppError>;

    fn handle(&mut self, message: WebRtcMessage, context: &mut Self::Context) -> Self::Result {
        WebRtcConnection::send_message(self.encoding, message, self, context)?;

        Ok(())
    }
}

impl Handler<RtcAnswerConnectionMessage> for WebRtcConnection {
    type Result = Result<(), AppError>;

    fn handle(
        &mut self,
        message: RtcAnswerConnectionMessage,
        context: &mut Self::Context,
    ) -> Self::Result {
        let peer_connection = Arc::clone(&self.peer_connection);
        let data_channel = self.data_channel_for_reader.clone();
        let connection_id = self.id;
        let address = context.address();
        let Ok(sdp) = RTCSessionDescription::answer(message.sdp) else {
            return Err(AppErrorTemplate::BadRequest(None).into());
        };

        async move { peer_connection.set_remote_description(sdp).await }
            .into_actor(self)
            .map(move |result: Result<(), webrtc::Error>, connection, _| {
                if result.is_err() {
                    connection
                        .web_socket_connection
                        .do_send(web_socket::message::CloseConnectionMessage);
                }
            })
            .spawn(context);

        let data_channel_listener = async move {
            let mut interval = time::interval(Duration::from_nanos(1));

            loop {
                interval.tick().await;

                let Ok(data_channel) = data_channel.lock() else {
                    continue;
                };
                let Some(data_channel) = data_channel.clone() else {
                    continue;
                };

                if Self::receive_message(data_channel, connection_id, address.clone())
                    .await
                    .is_err()
                {
                    return;
                };
            }
        };

        data_channel_listener
            .into_actor(self)
            .map(|_, _, context| context.address().do_send(CloseConnectionMessage))
            .spawn(context);

        self.heartbeat(context);

        Ok(())
    }
}

// impl Handler<RtcCandidateConnectionMessage> for WebRtcConnection {
//     type Result = Result<(), AppError>;
//
//     fn handle(
//         &mut self,
//         message: RtcCandidateConnectionMessage,
//         context: &mut Self::Context,
//     ) -> Self::Result {
//         let peer_connection = Arc::clone(&self.peer_connection);
//         let candidate = RTCIceCandidateInit {
//             candidate: message.candidate,
//             sdp_mid: message.sdp_mid,
//             sdp_mline_index: message.sdp_m_line_index,
//             username_fragment: message.username_fragment,
//         };
//         //
//         async move { peer_connection.add_ice_candidate(candidate).await }
//             .into_actor(self)
//             .map(|_, _, _| {})
//             .spawn(context);
//
//         Ok(())
//     }
// }

impl Handler<HelloConnectionMessage> for WebRtcConnection {
    type Result = Result<(), AppError>;

    fn handle(&mut self, _: HelloConnectionMessage, context: &mut Self::Context) -> Self::Result {
        let message = WebRtcMessage {
            id: -1,
            connection_id: self.id,
            opcode: Opcode::Hello,
            payload: WebRtcMessagePayload::Hello {
                user_id: self.registered_user_id.to_string(),
                users: User::find_all_by_room_id(&self.registered_room_id)?
                    .iter()
                    .map(|user| user.clone().into())
                    .collect(),
                messages: Message::find_all_by_room_id(&self.registered_room_id)?
                    .iter()
                    .map(|message| message.clone().into())
                    .collect(),
            },
        };
        Self::send_message(self.encoding, message, self, context)
    }
}

impl Handler<SendToServiceHandlerConnectionMessage> for WebRtcConnection {
    type Result = Result<(), AppError>;

    fn handle(
        &mut self,
        message: SendToServiceHandlerConnectionMessage,
        context: &mut Self::Context,
    ) -> Self::Result {
        WebRtc::handle_message(self, message.message, context)
    }
}

impl Handler<CloseConnectionMessage> for WebRtcConnection {
    type Result = Result<(), AppError>;

    fn handle(&mut self, _: CloseConnectionMessage, context: &mut Self::Context) -> Self::Result {
        let peer_connection = self.peer_connection.clone();

        async move {
            let _ = peer_connection.close().await;
        }
        .into_actor(self)
        .map(|_, _, context| {
            context.stop();
        })
        .spawn(context);

        Ok(())
    }
}
