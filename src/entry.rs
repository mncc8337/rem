use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Entry {
    pub name: String,
    pub interval: u64,
    pub message: String,
    pub urgency: u8,
    pub icon: String,
    pub creation_time: u128,
}

impl Entry {
    pub fn new(name: String, interval: u64, message: String, urgency: u8, icon: String) -> Self {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self {
            name,
            interval,
            message,
            urgency,
            icon,
            creation_time: since_epoch.as_millis(),
        }
    }
}
