use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: String,
    pub port: i32,
    pub passkey: String,
    #[serde(default)]
    pub action: Option<Action>,
    #[serde(default)]
    pub num_want: Option<isize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Completed,
    Started,
    Stopped,
}