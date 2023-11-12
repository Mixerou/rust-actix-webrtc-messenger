use nanoid::nanoid;
use structsy::derive::{queries, Persistent};
use structsy::StructsyTx;

use crate::database;
use crate::error::{AppError, AppErrorTemplate};
use crate::utils::snowflake_generator;

#[queries(Session)]
trait SessionQueries {
    fn filter_by_token(self, token: String) -> Self;
}

#[derive(Debug, Persistent)]
pub struct Session {
    #[index(mode = "exclusive")]
    pub id: i64,
    #[index(mode = "exclusive")]
    pub token: String,
}

impl Session {
    pub fn create() -> Result<Self, AppError> {
        let database = database::get();
        let mut transaction = database.begin()?;

        let session = Self {
            id: snowflake_generator::generate(),
            token: format!("{}{}", nanoid!(45), snowflake_generator::generate()),
        };

        transaction.insert(&session)?;
        transaction.commit()?;

        Ok(session)
    }

    pub fn find_by_token(token: &str) -> Result<Self, AppError> {
        let database = database::get();

        if let Some((_, session)) = database
            .query::<Self>()
            .filter_by_token(token.to_string())
            .into_iter()
            .next()
        {
            return Ok(session);
        }

        Err(AppErrorTemplate::NotFound(None).into())
    }
}
