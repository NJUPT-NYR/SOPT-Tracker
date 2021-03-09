use super::*;
use std::{
    convert::TryFrom,
    net::{Ipv4Addr, Ipv6Addr},
};
mod seedermap;
use seedermap::SeederMap;

type Key = u64;
type Value = PeerInfo;

pub struct Bucket {
    time_to_compaction: u64,
    key: Key,
    value: Value,
}

pub enum SeederInfo {
    NoneSeeder,
    OneSeeder([Bucket; 1]),
    TwoSeeder([Bucket; 2]),
    ThreeSeeder([Bucket; 3]),
    MulitSeeder(SeederMap),
}

impl SeederInfo {
    pub fn new() -> Self {
        SeederInfo::NoneSeeder
    }

    pub fn compaction(&mut self) {
        match self {
            SeederInfo::MulitSeeder(sm) => sm.compaction(),
            _ => todo!(),
        }
    }

    pub fn gen_response(&self, num_want: usize) -> RedisValue {
        match self {
            SeederInfo::MulitSeeder(sm) => sm.gen_response(num_want),
            _ => todo!(),
        }
    }
    pub fn update(&mut self, uid: u64, p: PeerInfo) {
        match self {
            SeederInfo::MulitSeeder(sm) => sm.update(uid, p),
            _ => todo!(),
        }
    }
}
