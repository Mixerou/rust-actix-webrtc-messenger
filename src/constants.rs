use std::time::Duration;

// Actors
pub const WEB_SOCKET_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
pub const WEB_SOCKET_CLIENT_TIMEOUT: Duration = Duration::from_secs(45);
pub const WEB_RTC_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub const WEB_RTC_CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
pub const WEB_RTC_DATA_CHANNEL_BUFFER_SIZE: usize = 1024 * 4;

// Models
pub const MESSAGE_CONTENT_MIN_LENGTH: usize = 1;
pub const MESSAGE_CONTENT_MAX_LENGTH: usize = 1024;
pub const ROOM_NAME_MIN_LENGTH: usize = 3;
pub const ROOM_NAME_MAX_LENGTH: usize = 32;
pub const USER_USERNAME_MIN_LENGTH: usize = 3;
pub const USER_USERNAME_MAX_LENGTH: usize = 32;
