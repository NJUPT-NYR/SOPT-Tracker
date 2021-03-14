use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use deadpool_redis::{cmd, Cmd};
use error::ProxyError;
use serde::{Deserialize};

use crate::error;

#[derive(Deserialize, Debug)]
pub struct AnnounceRequestData {
    // deprecated
    // pub info_hash: Vec<u8>,
    pub peer_id: String,
    pub port: u16,
    pub uid: u64,
    pub tid: u64,
    pub passkey: String,
    pub ip: Option<IpAddr>,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    #[serde(default)]
    pub event: Event,
    #[serde(default = "crate::config::default_num_want")]
    pub numwant: u16,
}

impl AnnounceRequestData {
    pub fn validation(&mut self) -> Result<(), error::ProxyError> {
        if self.peer_id.len() != 20 {
            println!("peerid {}", self.peer_id.len());
            return Err(ProxyError {});
        }
        Ok(())
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
            panic!("unable to detect connection address");
        }
        self.ipv4 = true_v4;
        self.ipv6 = true_v6;
    }

    pub fn into_announce_cmd(&self) -> Cmd {
        let ipv4 = match self.ipv4 {
            Some(ip) => ip.to_string(),
            None => String::from("none"),
        };
        let ipv6 = match self.ipv6 {
            Some(ip) => ip.to_string(),
            None => String::from("none"),
        };
        let mut acmd = cmd("ANNOUNCE");
        acmd.arg(self.tid)
            .arg(self.uid)
            .arg(ipv4)
            .arg(ipv6)
            .arg(self.port)
            .arg(self.numwant)
            .arg(self.event.to_string());
        acmd
    }
}
#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Event {
    Started = 0,
    Completed = 1,
    Stopped = 2,
}

impl Default for Event {
    fn default() -> Self {
        Event::Started
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        match self {
            Event::Started => "started",
            Event::Completed => "completed",
            Event::Stopped => "stopped",
        }
        .into()
    }
}
