use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub interval: u64,
    pub message: String,
    pub enabled: bool,
}
