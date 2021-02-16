use std::mem::size_of;

use actix_web::{web, HttpRequest};
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub port: i32,
    pub passkey: String,
    pub ip: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    #[serde(default)]
    pub event: Event,
    #[serde(default = "crate::app::config::default_num_want")]
    pub numwant: isize,
}

impl AnnounceRequestData {
    fn check_validation(&mut self) -> bool {
        if self.info_hash.len() != 20 {
            return false;
        }
        if self.peer_id.len() != 20 {
            return false;
        }
        true
    }
}

#[repr(C)]
pub struct AnnouncePacket {
    version: u8,
    event: u8,
    numwant: u16,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    v4ip: [u8; 4],
    v6ip: [u16; 8],
    port: u16,
    draft: [u8; 14],
}

impl AnnouncePacket {
    fn to_stream(&self) {}
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Event {
    Completed,
    Started,
    Stopped,
}

impl Default for Event {
    fn default() -> Self {
        Event::Started
    }
}
