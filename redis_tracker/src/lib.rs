#[macro_use]
extern crate redis_module;

use peerinfo::PeerInfo;
use redis_module::{native_types::RedisType, Status};
use redis_module::{raw, Context, RedisError, RedisResult, RedisValue};
use seederinfo::SeederInfo;
use std::convert::TryFrom;
use std::os::raw::c_void;
use std::time::Duration;

mod peerinfo;
mod seederinfo;
mod util;

struct AnnounceRequest {
    pid: u64,
    uid: u64,
    peer: PeerInfo,
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
    Box::from_raw(value as *mut SeederInfo);
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
        let peer = PeerInfo::from(ipv4, ipv6, port);
        return Ok(Self { pid, uid, peer });
    }
}

/* ANNOUNCE <pid> <uid> <v4ip> <v6ip> <port> <EVENT> <NUMWANT> */
fn announce(ctx: &Context, args: Vec<String>) -> RedisResult {
    let AnnounceRequest { pid, uid, peer } = AnnounceRequest::try_from(args)?;
    let num_want = 50;
    let key = ctx.open_key_writable(pid.to_string().as_str());
    if key.is_empty() {
        let value = SeederInfo::new();
        key.set_value(&SEEDER_MAP_TYPE, value)?;
    }

    let sm: &mut SeederInfo;
    sm = match key.get_value::<SeederInfo>(&SEEDER_MAP_TYPE)? {
        Some(value) => value,
        None => return Err(RedisError::Str("FUCK U")),
    };
    sm.compaction();
    sm.update(uid, peer);
    key.set_expire(Duration::from_secs(2700))?;
    Ok(sm.gen_response(num_want))
}

fn init(_: &Context, _: &Vec<String>) -> Status {
    // ctx.log_notice(format!("PeerInfo {}", std::mem::size_of::<PeerInfo>()).as_str());
    // ctx.log_notice(format!("PeerInfo_O {}", std::mem::size_of::<peerinfo::PeerInfo_O>()).as_str());
    // ctx.log_notice(format!("Bucket {}", std::mem::size_of::<seederinfo::Bucket>()).as_str());
    // ctx.log_notice(format!("SeederMap {}", std::mem::size_of::<seederinfo::SeederMap>()).as_str());
    // ctx.log_notice(format!("SeederInfo {}", std::mem::size_of::<SeederInfo>()).as_str());
    // ctx.log_notice(format!("Bucket {}", std::mem::size_of::<Bucket>()).as_str());
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
    // use super::*;

    #[test]
    fn check_struct_size() {
        // println!("{}", std::mem::size_of::<PeerInfo>());
        // println!("{}", std::mem::size_of::<seederinfo::Bucket>());
    }
}
