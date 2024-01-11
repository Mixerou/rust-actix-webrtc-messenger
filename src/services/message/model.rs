use actix::SystemService;
use serde::{Deserialize, Serialize};
use structsy::derive::{Persistent, queries};
use structsy::StructsyTx;

use crate::constants::{MESSAGE_CONTENT_MAX_LENGTH, MESSAGE_CONTENT_MIN_LENGTH};
use crate::database;
use crate::error::{AppError, AppErrorTemplate};
use crate::utils::snowflake_generator;
use crate::web_rtc::actor::WebRtc;
use crate::web_rtc::message::MessageUpdateMessage;

#[queries(Message)]
trait MessageQueries {
    fn filter_by_room_id(self, room_id: i64) -> Self;
}

#[derive(Clone, Debug, Persistent)]
pub struct Message {
    #[index(mode = "exclusive")]
    pub id: i64,
    pub author_id: i64,
    pub room_id: i64,
    pub content: String,
}

impl Message {
    pub fn create(author_id: i64, room_id: i64, content: String) -> Result<Self, AppError> {
        let database = database::get();
        let mut transaction = database.begin()?;

        let message = Self {
            id: snowflake_generator::generate(),
            author_id,
            room_id,
            content,
        };

        transaction.insert(&message)?;
        transaction.commit()?;

        WebRtc::from_registry().do_send(MessageUpdateMessage {
            message: message.clone(),
            room_id,
        });

        Ok(message)
    }

    pub fn find_all_by_room_id(id: &i64) -> Result<Vec<Self>, AppError> {
        let database = database::get();

        let messages = database
            .query::<Self>()
            .filter_by_room_id(*id)
            .into_iter()
            .map(|data| data.1)
            .collect();

        Ok(messages)
    }

    pub fn check_content_length(content: &str) -> Result<(), AppError> {
        let length = content.chars().count();

        match length {
            length if length < MESSAGE_CONTENT_MIN_LENGTH => {
                Err(AppErrorTemplate::MessageContentTooShort(None).into())
            }
            length if length > MESSAGE_CONTENT_MAX_LENGTH => {
                Err(AppErrorTemplate::MessageContentTooLong(None).into())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct MessagePublic {
    pub id: String,
    pub author_id: String,
    pub content: String,
}

impl From<Message> for MessagePublic {
    fn from(message: Message) -> Self {
        Self {
            id: message.id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.content,
        }
    }
}
