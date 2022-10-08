use crate::{
    config::{self, WriteMissPolicy::*, WritePolicy::*},
    utils::bits,
    lru::LRUSet,
    memory::QueryResult,
};

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: QueryResult,
    pub writeback: Option<u32>,
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

pub struct CPUCache {
    sets: Vec<LRUSet<CacheEntry>>,
    config: config::CacheConfig,
}

impl CPUCache {
    pub fn new(config: config::CacheConfig) -> Self {
        let search_function = |tag: u32, entry: CacheEntry| entry.tag == tag;
        let empty_set = LRUSet::new(search_function, config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        CPUCache { 
            sets,
            config,
        }
    }

    /// Performs a read access to the cache
    pub fn read(&mut self, addr: u32) -> CacheResponse {
        let (block_addr, _block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];

        let (result, writeback) = match set.lookup(tag) {
            // Some block found: Hit
            Some(_block) => {
                (QueryResult::Hit, None)
            },
            // No block found: Miss
            None => {
                let new_entry = CacheEntry {
                    tag,
                    addr,
                    dirty: false,
                };
                let evicted_block = set.push(new_entry);
                let writeback = evicted_block
                    .filter(|block| {
                        block.is_dirty()
                    })
                    .map(|block| block.addr);
                (QueryResult::Miss, writeback)
            },
        };

        CacheResponse {
            tag,
            idx,
            result,
            writeback,
        }
    }

    /// Performs a write access to the cache according to the write policy.
    pub fn write(&mut self, addr: u32) -> CacheResponse {
        let (block_addr, _block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];
        let (result, writeback) = match set.lookup(tag) {
            // Some block found: Hit
            Some(block) => {
                if self.config.write_policy == WriteBack {
                    block.borrow_mut().enfilthen();
                }
                (QueryResult::Hit, None)
            },
            // No block found: Miss
            None => {
                if self.config.write_miss_policy == NoWriteAllocate {
                    (QueryResult::Miss, None)
                } else {
                    let new_entry = CacheEntry {
                        tag,
                        addr,
                        dirty: false,
                    };
                    let evicted_block = set.push(new_entry);
                    let writeback = evicted_block
                        .filter(|block| {
                            block.is_dirty()
                        })
                        .map(|block| block.addr);
                    (QueryResult::Miss, writeback)
                }
            },
        };

        CacheResponse {
            tag,
            idx,
            result,
            writeback,
        }
    }

    pub fn invalidate() {

    }

    /*
    /// Unconditionally inserts an entry into its corresponding set.
    pub fn writeback(&mut self, addr: u32) {
        unimplemented!()
    }
    */
}