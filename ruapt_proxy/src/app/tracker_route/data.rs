use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub num_want: i32,
    pub ip: String,
    pub port: i32,
    pub action: Option<Action>,
    pub passkey: Option<String>
}


#[derive(Serialize, Debug)]
pub struct AnnounceCallRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: String,
    pub port: i32,
    pub action: Action,
    pub num_want: isize,
}
impl AnnounceCallRequestData {
    pub fn encode_info(&self) -> String {
        format!("{}:{}:{}", self.peer_id, self.ip, self.port)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Completed,
    Started,
    Stopped,
}

#[derive(Serialize, Debug)]
pub struct Peer {
    peer_id: String,
    ip: String,
    port: i32,
}

impl Peer {
    pub fn from(info: &String) -> Peer {
        let tmp: Vec<&str> = info.split(':').collect();
        Peer {
            peer_id: tmp[0].into(),
            ip: tmp[1].into(),
            port: tmp[2].parse().unwrap(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AnnounceResponseData {
    pub peers: Vec<Peer>,
}
