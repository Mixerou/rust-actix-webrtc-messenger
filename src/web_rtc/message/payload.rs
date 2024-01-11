use paste::paste;
use rmpv::{Utf8String, Value};
use serde::de::Error;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::payload_enum_helper;
use crate::services::message::model::MessagePublic;
use crate::services::user::model::UserPublic;

payload_enum_helper! {
    #[derive(Clone, Debug, Default)]
    enum WebRtcMessagePayload {
        // Opcode: Request
        RequestPostMessage { content: String, } = "10" | 10,

        // Opcode: Response
        Response { code: u32, message: String, } = "20" | 20,

        // Opcode: Dispatch
        DispatchUserUpdate {
            user: UserPublic,
        } = "40" | 40,
        DispatchMessageUpdate {
            message: MessagePublic,
        } = "41" | 41,

        // Opcode: Hello
        Hello {
            user_id: String,
            users: Vec<UserPublic>,
            messages: Vec<MessagePublic>,
        } = "50" | 50,

        // Other
        #[default]
        None = "0" | 0,
    }
}

impl WebRtcMessagePayload {
    pub fn is_none(&self) -> bool {
        matches!(self, WebRtcMessagePayload::None)
    }
}

impl<'de> Deserialize<'de> for WebRtcMessagePayload {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut values = Vec::new();
        let value = Value::deserialize(deserializer)?;
        let Some(payload) = value.as_map() else {
            return Err(Error::custom("Not a Map"));
        };
        let Some(payload_type) = &payload.first() else {
            return Err(Error::custom("There are no elements in payload"));
        };

        values.append(&mut payload.to_vec());

        values[0] = (
            Value::String(Utf8String::from("t")),
            Value::String(Utf8String::from(payload_type.1.to_string())),
        );

        Ok(WebRtcMessagePayloadHelper::deserialize(Value::Map(values))
            .unwrap_or_default()
            .into())
    }
}

impl Serialize for WebRtcMessagePayload {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut message: S::SerializeMap = serializer.serialize_map(None)?;

        self.serialize_fields::<S>(&mut message)?;
        message.end()
    }
}
