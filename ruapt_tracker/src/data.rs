use crate::error::*;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{collections::HashMap, mem::transmute_copy};

#[repr(C)]
#[derive(Debug)]
pub struct AnnouncePacket {
    version: u8,
    pub event: u8,
    pub numwant: u16,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
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
        unsafe { std::mem::transmute(self) }
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8; 80] {
        unsafe { std::mem::transmute(self) }
    }
}

impl AnnouncePacket {
    pub fn encode_info(&self) -> String {
        format!(
            "{}@{}@{}@{}",
            unsafe { std::str::from_utf8_unchecked(&self.peer_id) },
            self.v4ip,
            self.v6ip,
            self.port
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Event {
    started = 0,
    completed = 1,
    stopped = 2,
}

#[cfg(scrape = "on")]
#[derive(Deserialize, Debug)]
pub struct ScrapeRequestData {
    pub info_hash: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Peer {
    peer_id: Vec<u8>,
    ipv4: Vec<u8>,
    ipv6: Vec<u8>,
    port: i32,
}

impl Peer {
    pub fn from(info: &Vec<u8>) -> TrackerResult<Peer> {
        let tmp: Vec<&[u8]> = info.split(|&ch| ch as char == '@').collect();
        if let Some(p_sli) = tmp.get(3) {
            if let Ok(ps) = std::str::from_utf8(p_sli) {
                if let Ok(port) = ps.parse() {
                    return Ok(Peer {
                        peer_id: tmp[0].into(),
                        ipv4: tmp[1].into(),
                        ipv6: tmp[2].into(),
                        port,
                    });
                }
            }
        }
        Err(TrackerError::ParseError("Can not convert to Peer"))
    }
}

#[cfg(scrape = "on")]
#[derive(Serialize, Debug)]
pub struct TorrentInfo {
    complete: isize,
    incomplete: isize,
    downloaded: isize,
}

#[cfg(scrape = "on")]
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
#[cfg(scrape = "on")]
#[derive(Serialize, Debug)]
pub struct ScrapeResponseData {
    pub files: HashMap<String, TorrentInfo>,
}
