use std::time::SystemTime;

pub type Timestamp = u64;

pub fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as Timestamp
}

// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Timestamp {
//     pub millis: u64,
// }
//
// impl Timestamp {
//     pub const fn from_millis(millis: u64) -> Self {
//         Timestamp { millis }
//     }
//
//     pub fn now() -> Self {
//         let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
//         Timestamp { millis: now.as_millis() as u64 }
//     }
// }
//
// impl Into<u64> for Timestamp {
//     fn into(self) -> u64 {
//         self.millis
//     }
// }
