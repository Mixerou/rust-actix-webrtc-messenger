use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, WrapFuture,
};
use actix_web_actors::ws::WebsocketContext;

use crate::error::{AppError, AppErrorTemplate};
use crate::services::room::model::Room;
use crate::services::user::model::User;
use crate::web_rtc;
use crate::web_rtc::connection::WebRtcConnection;
use crate::web_socket::connection::WebSocketConnection;
use crate::web_socket::message::{WebSocketMessage, WebSocketMessagePayload};

pub fn get_sdp_offer(
    message: WebSocketMessage,
    connection: &mut WebSocketConnection,
    context: &mut WebsocketContext<WebSocketConnection>,
) -> Result<(), AppError> {
    let WebSocketMessagePayload::RequestGetRoomSdpOffer {
        room_name,
        username,
    } = message.payload
    else {
        return Err(AppErrorTemplate::BadRequest(None).into());
    };

    Room::check_name_length(&room_name)?;
    User::check_username_length(&username)?;

    if let (Some(room_id), Some(user_id)) = (
        &connection.registered_room_id,
        &connection.registered_user_id,
    ) {
        Room::unregister_connection(&connection.id, room_id, user_id)?;
    }

    let session_id = connection
        .session_id
        .ok_or_else(|| AppErrorTemplate::Unauthorized(None))?;
    let room = match Room::find_by_name(&room_name) {
        Ok(room) => room,
        Err(error) => {
            if error.http_code != 404 {
                return Err(error);
            }

            Room::create(room_name)?
        }
    };

    let user = match User::create(username.to_owned(), room.id, session_id) {
        Ok(user) => user,
        Err(error) => {
            if error.http_code != 409 {
                return Err(error);
            }

            let user = User::find_by_username_and_room_id(&username, &room.id)?;
            if user.session_id != session_id {
                return Err(AppErrorTemplate::UsernameTaken(None).into());
            }

            user
        }
    };

    connection.registered_room_id = Some(room.id);
    connection.registered_user_id = Some(user.id);
    Room::register_connection(connection.id, &room.id, &user.id)?;

    if let Ok(web_rtc_connection) = connection.web_rtc_connection.lock() {
        if let Some(ref web_rtc_connection) = *web_rtc_connection {
            web_rtc_connection.do_send(web_rtc::message::CloseConnectionMessage);
        }
    }

    let connection_id = connection.id;
    let connection_encoding = connection.encoding;
    let connection_address = context.address();

    async move {
        let Ok(web_rtc_connection) = WebRtcConnection::try_new(
            connection_id,
            connection_encoding,
            room.id,
            user.id,
            connection_address,
        )
        .await
        else {
            return Err(AppErrorTemplate::InternalServerError(None).into());
        };
        let web_rtc_connection = web_rtc_connection.start();

        Ok(web_rtc_connection)
    }
    .into_actor(connection)
    .map(
        move |result: Result<Addr<WebRtcConnection>, AppError>, connection, context| match result {
            Ok(address) => {
                if let Ok(mut web_rtc_connection) = connection.web_rtc_connection.lock() {
                    *web_rtc_connection = Some(address)
                }
            }
            Err(_) => context.stop(),
        },
    )
    .spawn(context);

    Ok(())
}

pub fn post_sdp_answer(
    message: WebSocketMessage,
    connection: &mut WebSocketConnection,
    _context: &mut WebsocketContext<WebSocketConnection>,
) -> Result<(), AppError> {
    let WebSocketMessagePayload::RequestPostRoomSdpAnswer { sdp } = message.payload else {
        return Err(AppErrorTemplate::BadRequest(None).into());
    };

    let Ok(web_rtc_connection) = connection.web_rtc_connection.lock() else {
        return Err(AppErrorTemplate::InternalServerError(None).into());
    };

    let Some(ref web_rtc_connection) = *web_rtc_connection else {
        return Err(AppErrorTemplate::WebRtcOfferNotRequested(None).into());
    };

    web_rtc_connection.do_send(web_rtc::message::RtcAnswerConnectionMessage { sdp });

    Ok(())
}

// pub fn post_ice_candidate(
//     message: WebSocketMessage,
//     connection: &mut WebSocketConnection,
//     _context: &mut WebsocketContext<WebSocketConnection>,
// ) -> Result<(), AppError> {
//     let WebSocketMessagePayload::RequestPostRoomIceCandidate {
//         candidate,
//         sdp_mid,
//         sdp_m_line_index,
//         username_fragment,
//     } = message.payload
//         else {
//             return Err(AppErrorTemplate::BadRequest(None).into());
//         };
//
//     let Some(ref web_rtc_connection) = *connection.web_rtc_connection else {
//         return Err(AppErrorTemplate::WebRtcOfferNotRequested(None).into());
//     };
//
//     web_rtc_connection.do_send(web_rtc::message::RtcCandidateConnectionMessage {
//         candidate,
//         sdp_mid,
//         sdp_m_line_index,
//         username_fragment,
//     });
//
//     Ok(())
// }
