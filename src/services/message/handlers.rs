use actix::Context;

use crate::error::{AppError, AppErrorTemplate};
use crate::services::message::model::Message;
use crate::web_rtc::connection::WebRtcConnection;
use crate::web_rtc::message::{WebRtcMessage, WebRtcMessagePayload};

pub fn post_message(
    message: WebRtcMessage,
    connection: &mut WebRtcConnection,
    _context: &mut Context<WebRtcConnection>,
) -> Result<(), AppError> {
    let WebRtcMessagePayload::RequestPostMessage { content } = message.payload else {
        return Err(AppErrorTemplate::BadRequest(None).into());
    };

    Message::check_content_length(&content)?;
    Message::create(
        connection.registered_user_id,
        connection.registered_room_id,
        content,
    )?;

    Ok(())
}
