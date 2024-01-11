use lazy_static::lazy_static;
use structsy::Structsy;

use crate::services::message::model::Message;
use crate::services::room::model::Room;
use crate::services::session::model::Session;
use crate::services::user::model::User;

lazy_static! {
    static ref DATABASE: Structsy = {
        let database = Structsy::memory().expect("Failed to open in-memory database");

        database
            .define::<Message>()
            .expect("Failed to define Message");
        database.define::<Room>().expect("Failed to define Room");
        database
            .define::<Session>()
            .expect("Failed to define Session");
        database.define::<User>().expect("Failed to define User");

        database
    };
}

pub fn get() -> &'static Structsy {
    &DATABASE
}

pub fn init() {
    info!("Initialize Database");

    lazy_static::initialize(&DATABASE);
}
