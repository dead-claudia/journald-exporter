use std::time::Duration;
use std::time::SystemTime;

pub fn mock_system_time(seconds: u64, millis: u32) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::new(seconds, millis * 1000000)
}
