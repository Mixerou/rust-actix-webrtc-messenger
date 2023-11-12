use actix_web_actors::ws::WebsocketContext;

use crate::error::{AppError, AppErrorTemplate};
use crate::services::room::model::Room;
use crate::services::user::model::User;
use crate::web_socket::actor::WebSocket;
use crate::web_socket::connection::WebSocketConnection;
use crate::web_socket::message::{Opcode, WebSocketMessage, WebSocketMessagePayload};

pub fn get_rtc_offer(
    message: WebSocketMessage,
    connection: &mut WebSocketConnection,
    context: &mut WebsocketContext<WebSocketConnection>,
) -> Result<(), AppError> {
    let WebSocketMessagePayload::RequestGetRoomRtcOffer {
        room_name,
        username,
    } = message.payload
    else {
        return Err(AppErrorTemplate::BadRequest(None).into());
    };

    Room::check_name_length(&room_name)?;
    User::check_username_length(&username)?;

    if let Some(room_id) = connection.registered_room_id {
        Room::unregister_connection(&connection.id, &room_id)?;
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

    if let Err(error) = User::create(username.to_owned(), room.id, session_id) {
        if error.http_code != 409 {
            return Err(error);
        }

        if User::find_by_username_and_room_id(&username, &room.id)?.session_id != session_id {
            return Err(AppErrorTemplate::UsernameTaken(None).into());
        }
    };

    connection.registered_room_id = Some(room.id);
    Room::register_connection(connection.id, &room.id)?;

    // Response to request
    let response = WebSocketMessage {
        id: message.id,
        connection_id: connection.id,
        opcode: Opcode::Response,
        payload: WebSocketMessagePayload::ResponseRoomRtcOffer {
            connection_id: connection.id.to_string(),
            // TODO: Send real SDP
            sdp: "".into(),
        },
    };

    WebSocket::send_message(message.id, response, connection, context);

    Ok(())
}

pub fn post_rtc_answer(
    message: WebSocketMessage,
    _connection: &mut WebSocketConnection,
    _context: &mut WebsocketContext<WebSocketConnection>,
) -> Result<(), AppError> {
    let WebSocketMessagePayload::RequestPostRoomRtcAnswer { .. } = message.payload else {
        return Err(AppErrorTemplate::BadRequest(None).into());
    };

    Ok(())
}
