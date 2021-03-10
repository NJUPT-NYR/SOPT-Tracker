use super::*;
mod seederarray;
mod seedermap;
use peerinfo::PeerInfo;
use seederarray::SeederArray;
pub use seedermap::SeederMap;

type Key = u64;
type Value = PeerInfo;

#[derive(Clone)]
pub struct Bucket {
    time_to_compaction: u64,
    pub key: Key,
    pub value: Value,
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            time_to_compaction: 0,
            key: Default::default(),
            value: Default::default(),
        }
    }

    pub fn from(k: Key, v: Value) -> Self {
        Bucket {
            time_to_compaction: util::get_timestamp() + 2700,
            key: k,
            value: v,
        }
    }
}

impl Default for Bucket {
    fn default() -> Self {
        Self::new()
    }
}

pub enum SeederInfo {
    InlineSeeder(SeederArray),
    MulitSeeder(SeederMap),
}

impl SeederInfo {
    pub fn new() -> Self {
        SeederInfo::InlineSeeder(SeederArray::new())
    }

    pub fn compaction(&mut self) {
        match self {
            SeederInfo::InlineSeeder(sa) => sa.compaction(),
            SeederInfo::MulitSeeder(sm) => {
                sm.compaction();
                if sm.get_seeder_cnt() < 3 {
                    if let Ok(sa) = SeederArray::from(sm) {
                        *self = SeederInfo::InlineSeeder(sa);
                    }
                }
            }
        }
    }

    pub fn gen_response(&self, num_want: usize) -> RedisValue {
        match self {
            SeederInfo::MulitSeeder(sm) => sm.gen_response(num_want),
            SeederInfo::InlineSeeder(sa) => sa.gen_response(),
        }
    }

    pub fn update(&mut self, uid: u64, p: PeerInfo) {
        match self {
            SeederInfo::MulitSeeder(sm) => sm.update(uid, p),
            SeederInfo::InlineSeeder(sa) => {
                if let Err(_) = sa.insert(uid, &p) {
                    let mut sm = SeederMap::from(sa);
                    sm.update(uid, p);
                    *self = SeederInfo::MulitSeeder(sm);
                }
            }
        }
    }
}
