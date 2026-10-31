use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageKind {
    Ping,
    Echo,
    Broadcast,
    Multicast,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub kind: MessageKind,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(id: u64, kind: MessageKind, data: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            kind,
            data: data.into(),
        }
    }
}
