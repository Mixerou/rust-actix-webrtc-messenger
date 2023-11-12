use structsy::derive::{queries, Persistent};
use structsy::StructsyTx;

use crate::constants::{ROOM_NAME_MAX_LENGTH, ROOM_NAME_MIN_LENGTH};
use crate::database;
use crate::error::{AppError, AppErrorTemplate};
use crate::services::user::model::User;
use crate::utils::snowflake_generator;

#[queries(Room)]
trait RoomQueries {
    fn filter_by_id(self, id: i64) -> Self;
    fn filter_by_name(self, name: String) -> Self;
}

#[derive(Debug, Persistent)]
pub struct Room {
    #[index(mode = "exclusive")]
    pub id: i64,
    #[index(mode = "exclusive")]
    pub name: String,
    pub active_connection_ids: Vec<i64>,
}

impl Room {
    pub fn create(name: String) -> Result<Self, AppError> {
        let database = database::get();
        let mut transaction = database.begin()?;

        let room = Self {
            id: snowflake_generator::generate(),
            name,
            active_connection_ids: Vec::new(),
        };

        transaction.insert(&room)?;
        transaction.commit()?;

        Ok(room)
    }

    pub fn find_by_name(name: &str) -> Result<Self, AppError> {
        let database = database::get();

        if let Some((_, room)) = database
            .query::<Self>()
            .filter_by_name(name.to_string())
            .into_iter()
            .next()
        {
            return Ok(room);
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn register_connection(id: i64, room_id: &i64) -> Result<(), AppError> {
        let database = database::get();

        if let Some((room_id, room)) = database
            .query::<Self>()
            .filter_by_id(*room_id)
            .into_iter()
            .next()
        {
            let mut transaction = database.begin()?;
            let mut room = room;

            room.active_connection_ids.push(id);
            transaction.update(&room_id, &room)?;
            transaction.commit()?;

            return Ok(());
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn unregister_connection(id: &i64, room_id: &i64) -> Result<(), AppError> {
        let database = database::get();

        if let Some((room_id, room)) = database
            .query::<Self>()
            .filter_by_id(*room_id)
            .into_iter()
            .next()
        {
            let mut transaction = database.begin()?;
            let mut room = room;

            room.active_connection_ids
                .retain(|&active_id| &active_id != id);

            match room.active_connection_ids.is_empty() {
                true => {
                    User::delete_by_room_id(&room.id)?;
                    transaction.delete(&room_id)?
                }
                false => transaction.update(&room_id, &room)?,
            }

            transaction.commit()?;

            return Ok(());
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }

    pub fn check_name_length(name: &str) -> Result<(), AppError> {
        let length = name.chars().count();

        match length {
            length if length < ROOM_NAME_MIN_LENGTH => {
                Err(AppErrorTemplate::RoomNameTooShort(None).into())
            }
            length if length > ROOM_NAME_MAX_LENGTH => {
                Err(AppErrorTemplate::RoomNameTooLong(None).into())
            }
            _ => Ok(()),
        }
    }
}
