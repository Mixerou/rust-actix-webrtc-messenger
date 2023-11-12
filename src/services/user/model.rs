use structsy::derive::{queries, Persistent};
use structsy::StructsyTx;

use crate::constants::{USER_USERNAME_MAX_LENGTH, USER_USERNAME_MIN_LENGTH};
use crate::database;
use crate::error::{AppError, AppErrorTemplate};
use crate::utils::snowflake_generator;

#[queries(User)]
trait UserQueries {
    fn filter_by_username(self, username: String) -> Self;
    fn filter_by_room_id(self, room_id: i64) -> Self;
}

#[derive(Debug, Persistent)]
pub struct User {
    #[index(mode = "exclusive")]
    pub id: i64,
    pub username: String,
    pub room_id: i64,
    pub session_id: i64,
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
        };

        transaction.insert(&user)?;
        transaction.commit()?;

        Ok(user)
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
