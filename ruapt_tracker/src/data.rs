use crate::error::*;
use bendy::encoding::{AsString, Error, SingleItemEncoder, ToBencode};
use serde::{Deserialize, Serialize};
use std::mem::transmute_copy;
use std::{
    convert::TryInto,
    net::{Ipv4Addr, Ipv6Addr},
};

#[repr(C)]
#[derive(Debug)]
pub struct AnnouncePacket {
    version: u8,
    pub event: u8,
    pub numwant: u16,
    pub info_hash: [u8; 20],
    pub passkey: [u8; 32],
    v4ip: Ipv4Addr,
    v6ip: Ipv6Addr,
    port: u16,
    draft: [u8; 2],
}

impl AnnouncePacket {
    pub fn new() -> Self {
        AnnouncePacket {
            version: 0,
            event: 0,
            numwant: 0,
            info_hash: [0; 20],
            passkey: [0; 32],
            v4ip: Ipv4Addr::UNSPECIFIED,
            v6ip: Ipv6Addr::UNSPECIFIED,
            port: 0,
            draft: [0; 2],
        }
    }

    #[allow(dead_code)]
    pub fn init_from_buffer(buf: &[u8; 80]) -> Self {
        unsafe { transmute_copy(buf) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8; 80] {
        unsafe { std::mem::transmute(self) }
    }
}

impl AnnouncePacket {
    pub fn encode_info(&self) -> [u8; 22] {
        let mut ret = [0u8; 22];
        let v4 = self.v4ip.octets();
        let v6 = self.v6ip.octets();
        let port = self.port.to_be_bytes();
        ret[0..4].copy_from_slice(&v4);
        ret[4..20].copy_from_slice(&v6);
        ret[20..].copy_from_slice(&port);
        ret
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Event {
    Started = 0,
    Completed = 1,
    Stopped = 2,
}

#[cfg(scrape = "on")]
#[derive(Deserialize, Debug)]
pub struct ScrapeRequestData {
    pub info_hash: Vec<Vec<u8>>,
}

#[derive(Serialize, Debug)]
pub struct Peer {
    peer4: Option<[u8; 6]>,
    peer6: Option<[u8; 18]>,
}

impl Peer {
    pub fn from(info: &Vec<u8>) -> TrackerResult<Peer> {
        if info.len() == 22 {
            let buf = info.as_slice();
            let (v4, v6_port) = buf.split_at(4);
            let (v6, port) = v6_port.split_at(16);
            let peer4 = if u32::from_le_bytes(v4.try_into().unwrap()) == 0 {
                None
            } else {
                let mut ret = [0u8; 6];
                ret[..4].copy_from_slice(&v4);
                ret[4..].copy_from_slice(&port);
                Some(ret)
            };
            let peer6 = if u128::from_le_bytes(v6.try_into().unwrap()) == 0 {
                None
            } else {
                let mut ret = [0u8; 18];
                ret[..16].copy_from_slice(&v6);
                ret[16..].copy_from_slice(&port);
                Some(ret)
            };
            Ok(Peer { peer4, peer6 })
        } else {
            Err(TrackerError::ParseError("Can not convert to Peer"))
        }
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
    peers: Vec<Peer>,
}

impl AnnounceResponseData {
    pub fn new(peers: Vec<Peer>) -> Self {
        Self { peers }
    }

    fn get_v4_peers(&self) -> Vec<u8> {
        let mut ret = vec![];
        for ref x in self.peers.iter().filter_map(|p| p.peer4) {
            ret.extend_from_slice(x);
        }
        ret
    }

    fn get_v6_peers(&self) -> Vec<u8> {
        let mut ret = vec![];
        for ref x in self.peers.iter().filter_map(|p| p.peer6) {
            ret.extend_from_slice(x);
        }
        ret
    }
}
#[cfg(scrape = "on")]
#[derive(Serialize, Debug)]
pub struct ScrapeResponseData {
    pub files: HashMap<String, TorrentInfo>,
}

impl ToBencode for AnnounceResponseData {
    const MAX_DEPTH: usize = 2;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"interval", 2700)?;
            e.emit_pair(b"peers", &AsString(&self.get_v4_peers()))?;
            e.emit_pair(b"peers6", &AsString(&self.get_v6_peers()))?;
            Ok(())
        })
    }
}
