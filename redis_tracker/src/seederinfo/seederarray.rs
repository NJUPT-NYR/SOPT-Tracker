use super::*;

pub struct SeederArray {
    seeders: [Bucket; 4],
    in_use: [bool; 4],
}

type SeederArrayIter<'a> = std::iter::Zip<std::slice::Iter<'a, Bucket>, std::slice::Iter<'a, bool>>;

impl SeederArray {
    pub fn new() -> Self {
        Self {
            seeders: Default::default(),
            in_use: [false; 4],
        }
    }

    pub fn iter(&self) -> SeederArrayIter {
        self.seeders.iter().zip(self.in_use.iter())
    }

    pub fn insert(&mut self, k: Key, v: &Value) -> Result<(), ()> {
        // try update
        for (b, &in_use) in self.seeders.iter_mut().zip(self.in_use.iter()) {
            if in_use && b.key == k {
                b.value = v.clone();
                b.time_to_compaction = util::get_timestamp() + 2700;
                return Ok(());
            }
        }
        // try push
        for (in_use, seeder) in self.in_use.iter_mut().zip(self.seeders.iter_mut()) {
            if *in_use == false {
                *seeder = Bucket::from(k, v.clone());
                *in_use = true;
                return Ok(());
            }
        }
        // overflow
        return Err(());
    }

    pub fn delete(&mut self, k: Key) {
        for (b, in_use) in self.seeders.iter().zip(self.in_use.iter_mut()) {
            if b.key == k {
                *in_use = false;
                return;
            }
        }
    }

    pub fn compaction(&mut self) {
        let now = util::get_timestamp();
        for (b, in_use) in self.seeders.iter().zip(self.in_use.iter_mut()) {
            if *in_use && now > b.time_to_compaction {
                *in_use = false;
            }
        }
    }

    pub fn gen_response(&self) -> RedisValue {
        let mut buf_peer: Vec<u8> = Vec::with_capacity(4 * 6);
        let mut buf_peer6: Vec<u8> = Vec::with_capacity(4 * 18);
        for (b, &in_use) in self.seeders.iter().zip(self.in_use.iter()) {
            if in_use {
                let p = &b.value;
                if let Some(ref v4) = p.get_ipv4() {
                    buf_peer.extend_from_slice(&v4.octets());
                    buf_peer.extend_from_slice(&p.get_port().to_be_bytes());
                };
                if let Some(v6) = p.get_ipv6() {
                    buf_peer6.extend_from_slice(&v6.octets());
                    buf_peer6.extend_from_slice(&p.get_port().to_be_bytes());
                };
            }
        }
        RedisValue::Array(vec![
            RedisValue::Buffer(buf_peer),
            RedisValue::Buffer(buf_peer6),
        ])
    }

    pub fn from(sm: &SeederMap) -> Result<Self, ()> {
        if sm.get_seeder_cnt() >= 3 {
            return Err(());
        }
        let mut sa = SeederArray::new();
        for (k,v) in sm.iter() {
            sa.insert(*k, v)?;
        }
        Ok(sa)
    }
}
