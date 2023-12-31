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
        RequestGetRoomRtcOffer { room_name: String, username: String, } = "10" | 10,
        RequestPostRoomRtcAnswer { sdp: String, } = "11" | 11,

        // Opcode: Response
        Response { code: u32, message: String, } = "20" | 20,
        ResponseSession { token: String, } = "21" | 21,
        ResponseRoomRtcOffer { connection_id: String, sdp: String, } = "22" | 22,

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

// impl From<Device> for WebSocketMessagePayload {
//     fn from(device: Device) -> Self {
//         let latest_data = match DeviceRecord::find_latest_by_device_id(device.id) {
//             Ok(record) => Some(record.data),
//             Err(_) => None,
//         };
//
//         WebSocketMessagePayload::DispatchDeviceUpdate {
//             id: device.id,
//             external_id: device.external_id,
//             name: device.name,
//             status: device.status,
//             kind: device.kind,
//             greenhouse_id: device.greenhouse_id,
//             created_at: device.created_at.duration_since(UNIX_EPOCH).unwrap().as_secs(),
//             maximum_data_value: device.maximum_data_value,
//             latest_data,
//         }
//     }
// }
