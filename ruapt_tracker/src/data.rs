use crate::error::*;
use serde::{Deserialize, Serialize};
pub use Action::*;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    // is it better to use info hash directly?
    // pub torrent_id: u64,
    pub ip: String,
    pub port: i32,
    pub action: Option<Action>,
    pub num_want: Option<isize>,
}

impl AnnounceRequestData {
    pub fn encode_info(&self) -> String {
        format!("{}@{}@{}", self.peer_id, self.ip, self.port)
    }
}

#[derive(Deserialize, Debug)]
pub enum Action {
    Completed,
    Started,
    Stopped,
}

#[derive(Deserialize, Debug)]
pub struct ScrapeRequestData {
    pub info_hashes: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub enum Request {
    Announce(AnnounceRequestData),
    Scrape(ScrapeRequestData),
}

#[derive(Serialize, Debug)]
pub struct Peer {
    peer_id: Vec<u8>,
    ip: Vec<u8>,
    port: i32,
}

impl Peer {
    pub fn from(info: &Vec<u8>) -> TrackerResult<Peer> {
        let tmp: Vec<&[u8]> = info.split(|&ch| ch as char == '@').collect();
        if let Some(p_sli) = tmp.get(2) {
            if let Ok(ps) = std::str::from_utf8(p_sli) {
                if let Ok(port) = ps.parse() {
                    return Ok(Peer {
                        peer_id: tmp[0].into(),
                        ip: tmp[1].into(),
                        port,
                    });
                }
            }
        }
        Err(TrackerError::ParseError("Can not convert to Peer"))
    }
}

#[derive(Serialize, Debug)]
pub struct TorrentInfo {
    complete: isize,
    incomplete: isize,
    downloaded: isize,
}

impl TorrentInfo {
    // no clue how to get download number for now.
    // TODO: add state for torrents
    pub fn new(complete: isize) -> Self {
        TorrentInfo {
            complete,
            incomplete: 0,
            downloaded: 114514,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AnnounceResponseData {
    pub peers: Vec<Peer>,
}

#[derive(Serialize, Debug)]
pub struct ScrapeResponseData {
    pub files: HashMap<String, TorrentInfo>,
}
