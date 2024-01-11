use paste::paste;
use rmpv::{Utf8String, Value};
use serde::de::Error;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::payload_enum_helper;

payload_enum_helper! {
    #[derive(Clone, Debug, Default)]
    enum WebSocketMessagePayload {
        // Opcode: Request
        RequestGetRoomSdpOffer { room_name: String, username: String, } = "10" | 10,
        RequestPostRoomSdpAnswer { sdp: String, } = "11" | 11,
        RequestPostRoomIceCandidate {
            candidate: String,
            sdp_mid: Option<String>,
            sdp_m_line_index: Option<u16>,
            username_fragment: Option<String>,
        } = "12" | 12, // Reserved

        // Opcode: Response
        Response { code: u32, message: String, } = "20" | 20,
        ResponseSession { token: String, } = "21" | 21,
        ResponseRoomRtcOffer { sdp: String, } = "22" | 22,

        // Opcode: Authorize
        Authorize { token: String, } = "30" | 30,

        // Other
        #[default]
        None = "0" | 0,
    }
}

impl WebSocketMessagePayload {
    pub fn is_none(&self) -> bool {
        matches!(self, WebSocketMessagePayload::None)
    }
}

impl<'de> Deserialize<'de> for WebSocketMessagePayload {
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

        Ok(
            WebSocketMessagePayloadHelper::deserialize(Value::Map(values))
                .unwrap_or_default()
                .into(),
        )
    }
}

impl Serialize for WebSocketMessagePayload {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut message: S::SerializeMap = serializer.serialize_map(None)?;

        self.serialize_fields::<S>(&mut message)?;
        message.end()
    }
}
