#[macro_use]
extern crate redis_module;

use indexmap::IndexMap;
use rand::Rng;
use redis_module::{native_types::RedisType, Status};
use redis_module::{raw, Context, RedisError, RedisResult, RedisValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    convert::TryFrom,
    net::{Ipv4Addr, Ipv6Addr},
};

use std::os::raw::c_void;

pub fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() & (std::u64::MAX - 1)
}

struct AnnounceRequest {
    pid: u64,
    uid: u64,
    peer: PeerInfo,
}

struct PeerInfo {
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
    port: u16,
}

type Key = u64;
type Value = PeerInfo;

type HashTable = IndexMap<Key, Value>;

struct Bucket {
    time_to_compaction: u64,
    key: Key,
    value: Value,
}

struct SeederMap {
    map: [HashTable; 2],
    time_to_compaction: u64,
}

enum SeederInfo {
    OneSeeder([Bucket; 1]),
    TwoSeeder([Bucket; 2]),
    ThreeSeeder([Bucket; 3]),
    MulitSeeder(SeederMap),
}

static SEEDER_MAP_TYPE: RedisType = RedisType::new(
    "SeederMap",
    0,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: None,
        rdb_save: None,
        aof_rewrite: None,
        free: Some(free),
        // Currently unused by Redis
        mem_usage: None,
        digest: None,
        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
    },
);

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

    fn update(&mut self, uid: u64, p: PeerInfo) {
        let m = self.get_mit_mut();
        m.insert(uid, p);
    }

    fn compaction(&mut self) {
        if get_timestamp() > self.time_to_compaction {
            let mit = self.get_mit_mut();
            *self.get_iit_mut() = IndexMap::with_capacity(mit.len() + 10);
            self.update_time_to_compaction();
            self.swap_mit()
        }
    }

    fn gen_response(&self, num_want: usize) -> RedisValue {
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

impl TryFrom<Vec<String>> for AnnounceRequest {
    type Error = RedisError;
    fn try_from(args: Vec<String>) -> Result<AnnounceRequest, RedisError> {
        if args.len() < 6 {
            return Err(RedisError::Str("FUCK U"));
        }
        let mut iter = args.into_iter().skip(1);
        let pid = iter.next().unwrap().parse::<u64>()?;
        let uid = iter.next().unwrap().parse::<u64>()?;
        let ipv4 = match iter.next().unwrap().as_str() {
            "none" => None,
            s @ _ => Some(s.parse()?),
        };
        let ipv6 = match iter.next().unwrap().as_str() {
            "none" => None,
            s @ _ => Some(s.parse()?),
        };
        let port: u16 = iter.next().unwrap().parse()?;
        let peer = PeerInfo { ipv4, ipv6, port };
        return Ok(Self { pid, uid, peer });
    }
}

/* ANNOUNCE <pid> <uid> <v4ip> <v6ip> <port> <EVENT> <NUMWANT> */
fn announce(ctx: &Context, args: Vec<String>) -> RedisResult {
    let AnnounceRequest { pid, uid, peer } = AnnounceRequest::try_from(args)?;
    let num_want = 50;
    let key = ctx.open_key_writable(pid.to_string().as_str());
    if key.is_empty() {
        let value = SeederMap::new();
        key.set_value(&SEEDER_MAP_TYPE, value)?;
    }

    let sm: &mut SeederMap;
    sm = match key.get_value::<SeederMap>(&SEEDER_MAP_TYPE)? {
        Some(value) => value,
        None => return Err(RedisError::Str("FUCK U")),
    };
    sm.compaction();
    sm.update(uid, peer);
    key.set_expire(Duration::from_secs(2700))?;
    Ok(sm.gen_response(num_want))
}

fn init(ctx: &Context, _: &Vec<String>) -> Status {
    // ctx.log(LogL, message)
    ctx.log_notice(format!("PeerInfo {}", std::mem::size_of::<PeerInfo>()).as_str());
    ctx.log_notice(format!("SeederMap {}", std::mem::size_of::<SeederMap>()).as_str());
    ctx.log_notice(format!("SeederInfo {}", std::mem::size_of::<SeederInfo>()).as_str());
    ctx.log_notice(format!("Bucket {}", std::mem::size_of::<Bucket>()).as_str());
    Status::Ok
}

redis_module! {
    name: "redistracker",
    version: 1,
    data_types: [SEEDER_MAP_TYPE],
    init: init,
    commands: [["announce", announce, "write deny-oom", 1, 1, 1]],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_struct_size() {
        println!("{}", std::mem::size_of::<PeerInfo>());
        println!("{}", std::mem::size_of::<SeederMap>());
        println!("{}", std::mem::size_of::<Bucket>());
    }
}
