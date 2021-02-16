use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: String,
    pub port: i32,
    pub passkey: String,
    #[serde(default)]
    pub action: Action,
    #[serde(default="crate::app::config::default_num_want")]
    pub num_want: isize,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Completed,
    Started,
    Stopped,
}

impl Default for Action {
    fn default() -> Self {
        Action::Started
    }
}