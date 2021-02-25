use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AnnounceRequestData {
    #[serde(with = "serde_bytes")]
    pub info_hash: Vec<u8>,
    pub peer_id: String,
    pub port: u16,
    pub passkey: String,
    pub ip: Option<IpAddr>,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    #[serde(default)]
    pub event: Event,
    #[serde(default = "crate::app::config::default_num_want")]
    pub numwant: u16,
}

impl AnnounceRequestData {
    pub fn check_validation(&mut self) -> bool {
        if self.info_hash.len() != 20 {
            println!("infohash {}", self.info_hash.len());
            return false;
        }
        if self.peer_id.len() != 20 {
            println!("peerid {}", self.info_hash.len());
            return false;
        }
        true
    }
    pub fn fix_ip(&mut self, peer_addr: Option<IpAddr>) {
        let mut true_v4 = None;
        let mut true_v6 = None;
        if let Some(ip) = self.ip {
            match ip {
                IpAddr::V4(v4) => true_v4 = self.ipv4.or(Some(v4)),
                IpAddr::V6(v6) => true_v6 = self.ipv6.or(Some(v6)),
            }
        }
        if let Some(ip) = peer_addr {
            match ip {
                IpAddr::V4(v4) => true_v4 = true_v4.or(Some(v4)),
                IpAddr::V6(v6) => true_v6 = true_v6.or(Some(v6)),
            }
        }
        if true_v4.is_none() && true_v6.is_none() {
            panic!("TODO");
        }
        self.ipv4 = true_v4;
        self.ipv6 = true_v6;
    }
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
    pub fn from(req: &AnnounceRequestData) -> Self {
        let mut p = AnnouncePacket {
            version: 1,
            event: req.event as u8,
            numwant: req.numwant,
            info_hash: [0; 20],
            peer_id: [0; 20],
            v4ip: req.ipv4.unwrap_or(Ipv4Addr::new(0, 0, 0, 0)),
            v6ip: req.ipv6.unwrap_or(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
            port: req.port,
            draft: [0; 14],
        };
        p.info_hash.copy_from_slice(req.info_hash.as_slice());
        p.peer_id.copy_from_slice(&req.peer_id.as_bytes()[..20]);
        p
    }

    pub fn as_bytes(&self) -> &[u8; 80] {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Event {
    started = 0,
    completed = 1,
    stopped = 2,
}

impl Default for Event {
    fn default() -> Self {
        Event::started
    }
}
