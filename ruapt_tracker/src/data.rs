use crate::error::*;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{collections::HashMap, mem::transmute_copy};
pub use Action::*;

#[derive(Deserialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub ip: String,
    pub port: i32,
    #[serde(default)]
    pub action: Option<Action>,
    #[serde(default)]
    pub num_want: Option<isize>,
}

#[repr(C)]
#[derive(Debug)]
pub struct AnnouncePacket {
    version: u8,
    event: u8,
    numwant: u16,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    v4ip: Ipv4Addr,
    v6ip: Ipv6Addr,
    port: u16,
    draft: [u8; 14],
}
impl AnnouncePacket {
    pub fn init_from_buffer(buf: &[u8; 80]) -> Self {
        unsafe { transmute_copy(buf) }
    }
    pub fn as_bytes(&self) -> &[u8; 80] {
        unsafe { std::mem::transmute(&self) }
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8; 80] {
        unsafe { std::mem::transmute(self) }
    }
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
    pub info_hash: Vec<String>,
}

#[serde(untagged)]
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
    pub fn new(complete: isize, incomplete: isize) -> Self {
        TorrentInfo {
            complete,
            incomplete,
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
