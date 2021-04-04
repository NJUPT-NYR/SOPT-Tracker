use bloom::CountingBloomFilter;
use std::collections::hash_map::HashMap;
use std::ops::DerefMut;
use std::sync::atomic::*;
use tokio::sync::RwLock;


enum Operation {
    Set,
    Delete,
}

/// A Filter built for solve both read and write in high concurrency
/// Updates will be operated in batch.
/// When a operation come, the update will be performed in 2 phases.
/// 1. add to cache
/// 2. once the cache is large enough, the updates will be commited.
///
/// And once the filter touch the capacity, in_expand will be set,
/// then, a new thread will be spawned to expand the capacity. meanwhile
/// the cache will not commit until the new filter build up.
pub struct Filter {
    inner: RwLock<CountingBloomFilter>,
    capacity: AtomicU32,
    cache: RwLock<HashMap<String, Operation>>,
    in_expand: AtomicBool,
}

impl Filter {
    fn batch_update(inner: &mut CountingBloomFilter, ops: HashMap<String, Operation>) {
        for (key, op) in ops.into_iter() {
            match op {
                Operation::Set => {
                    inner.insert_get_count(&key);
                }
                Operation::Delete => {
                    inner.remove(&key);
                }
            };
        }
    }

    async fn fetch_cache(&self) -> HashMap<String, Operation> {
        let mut new_cache = HashMap::new();
        let mut cache = self.cache.write().await;
        std::mem::swap(cache.deref_mut(), &mut new_cache);
        return new_cache;
    }

    pub fn new() -> Self {
        let capacity = AtomicU32::new(8192);
        let filter_inner = CountingBloomFilter::with_rate(4, 0.05, 8192);
        let inner = RwLock::new(filter_inner);
        let cache = RwLock::new(HashMap::new());
        let in_expand = AtomicBool::new(false);
        // let expand_thread = None;
        Self {
            inner,
            capacity,
            cache,
            in_expand,
            // expand_thread,
        }
    }

    pub async fn delete(&mut self, key: String) {
        let size;
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, Operation::Delete);
            size = cache.len();
        }
        if size > 64 {
            if self.in_expand.load(Ordering::Relaxed) == false {
                let cache = self.fetch_cache().await;
                let mut inner = self.inner.write().await;
                Self::batch_update(inner.deref_mut(), cache);
            }
        }
    }

    pub async fn insert(&mut self, key: String) {
        let size;
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, Operation::Set);
            size = cache.len();
        }
        if size > 64 {
            if self.in_expand.load(Ordering::Relaxed) == false {
                let cache = self.fetch_cache().await;
                let mut inner = self.inner.write().await;
                Self::batch_update(inner.deref_mut(), cache);
            }
        }
    }

    pub async fn contains(&self, key: &String) -> bool {
        let mut find;
        {
            let inner = self.inner.read().await;
            find = inner.estimate_count(key) > 0;
        }
        if !find {
            let cache = self.cache.read().await;
            find = matches!(cache.get(key), Some(Operation::Set));
        }
        return find;
    }

    // use some stream like, as Vec<String> is too large
    pub async fn check_expand(&self, keys: Vec<String>) {
        if keys.len() > self.capacity.load(Ordering::Relaxed) as usize {
            if self
                .in_expand
                .compare_and_swap(false, true, Ordering::Relaxed)
            {
                let new_cap = (keys.len() * 3 / 2) as u32;
                let mut new_filter = CountingBloomFilter::with_rate(4, 0.05, new_cap);
                self.capacity.store(new_cap, Ordering::Relaxed);
                {
                    // before expand, commit batch
                    let cache = self.fetch_cache().await;
                    let mut inner = self.inner.write().await;
                    Self::batch_update(inner.deref_mut(), cache);
                }
                for key in keys.into_iter() {
                    new_filter.insert_get_count(&key);
                }
                let cache = self.fetch_cache().await;
                for (key, op) in cache.into_iter() {
                    match op {
                        Operation::Set => {
                            new_filter.insert_get_count(&key);
                        }
                        Operation::Delete => {
                            // do nothing, as it might cause false negative
                        }
                    };
                }
                {
                    let mut inner = self.inner.write().await;
                    std::mem::swap(inner.deref_mut(), &mut new_filter);
                }
                self.in_expand.store(false, Ordering::Relaxed);
            }
        }
    }
}
