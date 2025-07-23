use std::time::SystemTime;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub millis: i64,
}

impl Timestamp {
    pub const fn from_millis(millis: i64) -> Self {
        Timestamp { millis }
    }

    pub fn now() -> Self {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Timestamp { millis: now.as_millis() as i64 }
    }
}