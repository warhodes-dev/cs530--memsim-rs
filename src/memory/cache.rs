use crate::{config, utils::bits};
use super::{QueryResult, AccessEvent};
use lru::LRUSet;

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: QueryResult,
    pub writeback: Option<u32>,
}

pub struct CPUCache {
    data: Vec<LRUSet>,
    config: config::CacheConfig,
}

impl CPUCache {
    pub fn new(config: config::CacheConfig) -> Self {
        let empty_set = LRUSet::new(config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        CPUCache { 
            data: sets,
            config,
        }
    }

    pub fn lookup(&mut self, access: AccessEvent) -> CacheResponse {
        let addr = access.addr();
        let (block_addr, block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.data[idx as usize];

        let (result, writeback) = match set.lookup(tag) {
            Some(block) => {
                if self.config.wback_enabled {
                    block.enfilthen();
                }
                (QueryResult::Hit, None)
            },
            None => {
                let new_entry = CacheEntry {
                    tag,
                    addr,
                    dirty: false,
                };
                let evicted_block = set.push(new_entry);
                let writeback = evicted_block
                    .filter(|block| {
                        block.is_dirty() && self.config.wback_enabled
                    })
                    .map(|block| {
                        block.addr
                    });

                (QueryResult::Miss, writeback)
            }

            // === HANDLED HERE
            // write back:
            //   when writing to the cache, simply flip dirty to true on that entry
            //   when an item is evicted from the cache:
            //     if it's clean, do nothing
            //     if it's dirty, write back to memory 
            // write through:
            //   when writing to the cache, also write to main memory
            //   TIP: everything is always clean

            // It's always WRITE BACK + WRITE ALLOCATE
        };
        CacheResponse {
            tag,
            idx,
            result,
            writeback,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CacheEntry {
    tag: u32,
    addr: u32,
    dirty: bool,
}

impl CacheEntry {
    fn enfilthen(&mut self) {
        self.dirty = true;
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

mod lru {
    use std::collections::VecDeque;
    use super::CacheEntry;

    #[derive(Clone, Debug)]
    pub struct LRUSet {
        inner: VecDeque<CacheEntry>,
        capacity: usize,
    }


    impl LRUSet {
        pub fn new(capacity: usize) -> Self {
            let inner = VecDeque::<CacheEntry>::with_capacity(capacity);
            LRUSet { inner, capacity }
        }

        pub fn push(&mut self, entry: CacheEntry) -> Option<CacheEntry> {
            let evicted_item = if self.inner.len() >= self.capacity {
                self.inner.pop_back()
            } else { None };
            self.inner.push_front(entry);
            evicted_item
        }

        pub fn lookup(&mut self, tag: u32) -> Option<&mut CacheEntry> {
            self.inner
                .iter()
                .position(|&entry| entry.tag == tag)
                .map(|item_idx| {
                    let item = self.inner.remove(item_idx).unwrap();
                    self.inner.push_front(item);
            });
            self.inner.iter_mut().last()
        }
    }
}