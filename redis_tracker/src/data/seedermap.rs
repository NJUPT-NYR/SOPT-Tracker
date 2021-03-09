use indexmap::IndexMap;
use rand::Rng;
use redis_module::{native_types::RedisType, Status};
use redis_module::{raw, Context, RedisError, RedisResult, RedisValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    convert::TryFrom,
    net::{Ipv4Addr, Ipv6Addr},
};

use super::*;
use std::os::raw::c_void;


pub fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() & (std::u64::MAX - 1)
}

pub struct AnnounceRequest {
    pid: u64,
    uid: u64,
    peer: PeerInfo,
}

type HashTable = IndexMap<Key, Value>;

pub struct SeederMap {
    map: [HashTable; 2],
    time_to_compaction: u64,
}

unsafe extern "C" fn free(value: *mut c_void) {
    Box::from_raw(value as *mut SeederMap);
}

impl SeederMap {
    fn new() -> Self {
        Self {
            map: [IndexMap::with_capacity(16), IndexMap::with_capacity(16)],
            time_to_compaction: (get_timestamp() + 2700),
        }
    }

    // mutable index table, can be 0/1
    fn mit(&self) -> u8 {
        (self.time_to_compaction % 2) as u8
    }

    fn swap_mit(&mut self) {
        self.time_to_compaction ^= 1;
    }

    fn get_mit(&self) -> &HashTable {
        &self.map[self.mit() as usize]
    }

    fn get_iit(&self) -> &HashTable {
        &self.map[(self.mit() ^ 1) as usize]
    }

    fn get_mit_mut(&mut self) -> &mut HashTable {
        &mut self.map[self.mit() as usize]
    }

    fn get_iit_mut(&mut self) -> &mut HashTable {
        &mut self.map[(self.mit() ^ 1) as usize]
    }

    fn update_time_to_compaction(&mut self) {
        let t = get_timestamp() + self.mit() as u64;
        self.time_to_compaction = t;
    }

    pub fn update(&mut self, uid: u64, p: PeerInfo) {
        let m = self.get_mit_mut();
        m.insert(uid, p);
    }

    pub fn compaction(&mut self) {
        if get_timestamp() > self.time_to_compaction {
            let mit = self.get_mit_mut();
            *self.get_iit_mut() = IndexMap::with_capacity(mit.len() + 10);
            self.update_time_to_compaction();
            self.swap_mit()
        }
    }

    pub fn gen_response(&self, num_want: usize) -> RedisValue {
        let mut buf_peer: Vec<u8> = Vec::with_capacity(num_want * 6);
        let mut buf_peer6: Vec<u8> = Vec::with_capacity(num_want * 18);
        let peer_cnt = self.map[0].len() + self.map[1].len();
        let max_right = if peer_cnt > num_want {
            peer_cnt - num_want
        } else {
            0
        };
        let rand = rand::thread_rng().gen_range(0..=max_right);
        let mut iter = self
            .get_mit()
            .iter()
            .chain(self.get_iit().iter())
            // is here O(n)?
            .skip(rand)
            .take(num_want);
        while let Some((_, p)) = iter.next() {
            if let Some(ref v4) = p.ipv4 {
                buf_peer.extend_from_slice(&v4.octets());
                buf_peer.extend_from_slice(&p.port.to_be_bytes());
            };
            if let Some(v6) = p.ipv6 {
                buf_peer6.extend_from_slice(&v6.octets());
                buf_peer6.extend_from_slice(&p.port.to_be_bytes());
            };
        }
        RedisValue::Array(vec![
            RedisValue::Buffer(buf_peer),
            RedisValue::Buffer(buf_peer6),
        ])
    }
}
