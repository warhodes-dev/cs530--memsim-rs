use crate::{config, utils::bits};
use super::{QueryResult, AccessEvent};
use lru::LRUSet;

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
    sets: Vec<LRUSet>,
    pub config: config::CacheConfig,
}

impl CPUCache {
    pub fn new(config: config::CacheConfig) -> Self {
        let empty_set = LRUSet::new(config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        CPUCache { 
            sets,
            config,
        }
    }

    pub fn lookup(&mut self, access: &AccessEvent) -> CacheResponse {

        let addr = access.addr();
        let (block_addr, block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];

        let (result, writeback) = match set.lookup(tag) {
            Some(block) => {
                if access.is_write() && self.config.wback_enabled {
                    block.borrow_mut().enfilthen();
                }
                (QueryResult::Hit, None)
            },
            None => {
                if access.is_write() && !self.config.walloc_enabled {
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
            }
        };

        CacheResponse {
            tag,
            idx,
            result,
            writeback,
        }
    }
}

mod lru {
    use std::{collections::VecDeque, cell::RefCell, rc::Rc};
    use super::CacheEntry;

    #[derive(Clone, Debug)]
    /// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
    //
    //  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
    //  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
    //  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
    pub struct LRUSet {
        inner: VecDeque<Rc<RefCell<CacheEntry>>>,
        capacity: usize,
    }

    impl LRUSet {
        pub fn new(capacity: usize) -> Self {
            let inner = VecDeque::<Rc<RefCell<CacheEntry>>>::with_capacity(capacity);
            LRUSet { inner, capacity }
        }

        /// Push an item to the LRU Set, potentially evicting the oldest item
        pub fn push(&mut self, entry: CacheEntry) -> Option<CacheEntry> {
            let evicted_item = if self.inner.len() >= self.capacity {
                self.inner.pop_back()
                    .map(|entry| {
                        entry.borrow().clone()
                    })
            } else { None };
            self.inner.push_front(Rc::new(RefCell::new(entry)));
            evicted_item
        }

        /// Look up an item in the LRU Set. If found, the item is 'touched' and moved to the front
        /// of the queue.
        pub fn lookup(&mut self, tag: u32) -> Option<Rc<RefCell<CacheEntry>>> {
            let item_search = self.inner
                .iter()
                .position(|entry| entry.borrow().tag == tag);
           
            if let Some(item_idx) = item_search {
                let item = self.inner.remove(item_idx).unwrap();
                self.inner.push_front(item.clone());
                Some(item)
            } else {
                None
            }
        }
    }
}