use indexmap::IndexMap;
use rand::Rng;
use redis_module::RedisValue;

use std::usize;
use util::*;

use super::*;

type HashTable = IndexMap<Key, Value>;

type SeederMapIter<'a> = std::iter::Chain<
    indexmap::map::Iter<'a, u64, peerinfo::PeerInfo>,
    indexmap::map::Iter<'a, u64, peerinfo::PeerInfo>,
>;

pub struct SeederMap {
    map: [HashTable; 2],
    time_to_compaction: u64,
    mit: u8,
    // draft: [u8; 7],
}

impl SeederMap {
    fn new() -> Self {
        Self {
            map: [IndexMap::with_capacity(16), IndexMap::with_capacity(16)],
            time_to_compaction: (util::get_timestamp() + 2700),
            mit: 0,
        }
    }

    pub fn from(sa: &SeederArray) -> Self {
        let mut t = Self::new();
        // future might need update
        for (b, in_use) in sa.iter() {
            if *in_use {
                t.update(b.key, b.value.clone())
            }
        }
        t
    }

    // mutable index table, can be 0/1
    fn mit(&self) -> u8 {
        self.mit
    }

    fn swap_mit(&mut self) {
        self.mit ^= 1;
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
        let t = get_timestamp();
        self.time_to_compaction = t;
    }

    pub fn get_seeder_cnt(&self) -> usize {
        self.get_iit().len() + self.get_mit().len()
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
            if let Some(ref v4) = p.get_ipv4() {
                buf_peer.extend_from_slice(&v4.octets());
                buf_peer.extend_from_slice(&p.get_port().to_be_bytes());
            };
            if let Some(v6) = p.get_ipv6() {
                buf_peer6.extend_from_slice(&v6.octets());
                buf_peer6.extend_from_slice(&p.get_port().to_be_bytes());
            };
        }
        RedisValue::Array(vec![
            RedisValue::Buffer(buf_peer),
            RedisValue::Buffer(buf_peer6),
        ])
    }

    pub fn iter(&self) -> SeederMapIter {
        self.get_mit().iter().chain(self.get_iit().iter())
    }
}
