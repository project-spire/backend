use std::time::SystemTime;

pub type Timestamp = u64;

pub fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as Timestamp
}
