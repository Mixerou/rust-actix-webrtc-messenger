use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

use lazy_static::lazy_static;
use snowflake::SnowflakeIdGenerator;

lazy_static! {
    static ref SNOWFLAKE_ID_GENERATOR: Mutex<SnowflakeIdGenerator> = {
        Mutex::new(SnowflakeIdGenerator::with_epoch(
            0,
            0,
            UNIX_EPOCH + Duration::from_millis(1696118400000),
        ))
    };
}

pub fn generate() -> i64 {
    SNOWFLAKE_ID_GENERATOR.lock().unwrap().real_time_generate()
}

pub fn init() {
    info!("Initialize Snowflake Generator");

    lazy_static::initialize(&SNOWFLAKE_ID_GENERATOR);
}
