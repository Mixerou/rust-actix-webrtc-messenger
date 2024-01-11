use actix::SystemService;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use structsy::derive::{queries, Persistent};
use structsy::StructsyTx;

use crate::constants::{USER_USERNAME_MAX_LENGTH, USER_USERNAME_MIN_LENGTH};
use crate::error::{AppError, AppErrorTemplate};
use crate::utils::snowflake_generator;
use crate::web_rtc::actor::WebRtc;
use crate::{database, web_rtc};

#[queries(User)]
trait UserQueries {
    fn filter_by_id(self, id: i64) -> Self;
    fn filter_by_username(self, username: String) -> Self;
    fn filter_by_room_id(self, room_id: i64) -> Self;
}

#[derive(Clone, Debug, Persistent)]
pub struct User {
    #[index(mode = "exclusive")]
    pub id: i64,
    pub username: String,
    pub room_id: i64,
    pub session_id: i64,
    pub active_connection_ids: Vec<i64>,
}

impl User {
    pub fn create(username: String, room_id: i64, session_id: i64) -> Result<Self, AppError> {
        let database = database::get();
        let mut transaction = database.begin()?;

        // Simulate unique by two columns
        if Self::find_by_username_and_room_id(&username, &room_id).is_ok() {
            return Err(AppErrorTemplate::Conflict(None).into());
        }

        let user = Self {
            id: snowflake_generator::generate(),
            username,
            room_id,
            session_id,
            active_connection_ids: Vec::new(),
        };

        transaction.insert(&user)?;
        transaction.commit()?;

        Ok(user)
    }

    pub fn find_all_by_room_id(id: &i64) -> Result<Vec<Self>, AppError> {
        let database = database::get();

        let users = database
            .query::<Self>()
            .filter_by_room_id(*id)
            .into_iter()
            .map(|data| data.1)
            .collect();

        Ok(users)
    }

    pub fn delete_by_room_id(id: &i64) -> Result<(), AppError> {
        let database = database::get();
        let mut transaction = database.begin()?;

        for (user_id, _) in database.query::<Self>().filter_by_room_id(*id).into_iter() {
            transaction.delete(&user_id)?;
        }

        transaction.commit()?;

        Ok(())
    }

    pub fn find_by_username_and_room_id(username: &str, room_id: &i64) -> Result<Self, AppError> {
        let database = database::get();

        if let Some((_, user)) = database
            .query::<Self>()
            .filter_by_username(username.to_string())
            .filter_by_room_id(*room_id)
            .into_iter()
            .next()
        {
            return Ok(user);
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn register_connection(id: i64, user_id: &i64) -> Result<(), AppError> {
        let database = database::get();

        if let Some((user_id, user)) = database
            .query::<Self>()
            .filter_by_id(*user_id)
            .into_iter()
            .next()
        {
            let mut transaction = database.begin()?;
            let mut user = user;
            let room_id = user.room_id;

            user.active_connection_ids.push(id);
            transaction.update(&user_id, &user)?;
            transaction.commit()?;

            if user.active_connection_ids.len() == 1 {
                WebRtc::from_registry()
                    .do_send(web_rtc::message::UserUpdateMessage { user, room_id });
            }

            return Ok(());
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn unregister_connection(id: &i64, user_id: &i64) -> Result<(), AppError> {
        let database = database::get();

        if let Some((user_id, user)) = database
            .query::<Self>()
            .filter_by_id(*user_id)
            .into_iter()
            .next()
        {
            let mut transaction = database.begin()?;
            let mut user = user;
            let room_id = user.room_id;

            user.active_connection_ids
                .retain(|&connection_id| &connection_id != id);

            transaction.update(&user_id, &user)?;
            transaction.commit()?;

            if user.active_connection_ids.is_empty() {
                WebRtc::from_registry()
                    .do_send(web_rtc::message::UserUpdateMessage { user, room_id });
            }

            return Ok(());
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn check_username_length(username: &str) -> Result<(), AppError> {
        let length = username.chars().count();

        match length {
            length if length < USER_USERNAME_MIN_LENGTH => {
                Err(AppErrorTemplate::UsernameTooShort(None).into())
            }
            length if length > USER_USERNAME_MAX_LENGTH => {
                Err(AppErrorTemplate::UsernameTooLong(None).into())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub status: UserStatus,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            status: match user.active_connection_ids.len() {
                0 => UserStatus::Offline,
                _ => UserStatus::Online,
            },
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum UserStatus {
    #[default]
    Offline = 0,
    Online = 1,
}
