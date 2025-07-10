use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Entry {
    pub name: String,
    pub interval: u64,
    pub message: String,
    pub timeout: i32,
    pub urgency: u8,
    pub icon: String,
}
