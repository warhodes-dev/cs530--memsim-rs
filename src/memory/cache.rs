use crate::{
    config::{self, WriteMissPolicy::*, WritePolicy::*},
    utils::bits,
    memory::QueryResult,
};

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: QueryResult,
    pub writeback: Option<u32>,
    pub eviction: Option<u32>,
}

#[derive(Copy, Clone, Debug)]
pub struct CacheEntry {
    tag: u32,
    addr: u32,
    ppn: u32,
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
    config: config::CacheConfig,
    global_config: config::Config,
}

impl CPUCache {
    pub fn new(config: config::CacheConfig, global_config: config::Config) -> Self {
        let empty_set = LRUSet::new(config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        CPUCache { sets, config, global_config, }
    }

    /// Performs a read access to the cache
    pub fn read(&mut self, addr: u32) -> CacheResponse {
        let (ppn, _page_offset) = bits::split_at(addr, self.global_config.pt.offset_size);
        let (block_addr, _block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];

        let (result, writeback, eviction) = match set.lookup(tag) {
            // Some block found: Hit
            Some(_block) => {
                (QueryResult::Hit, None, None)
            },
            // No block found: Miss
            None => {
                let new_entry = CacheEntry {
                    tag,
                    addr,
                    ppn,
                    dirty: false,
                };
                let evicted_block = set.push(new_entry);
                let evicted_addr = evicted_block.map(|b| b.addr);

                let writeback = evicted_block
                    .filter(|block| {
                        block.is_dirty()
                    })
                    .map(|block| block.addr);
                (QueryResult::Miss, writeback, evicted_addr)
            },
        };

        CacheResponse {
            tag,
            idx,
            result,
            writeback,
            eviction,
        }
    }

    pub fn write_force(&mut self, addr: u32) -> CacheResponse {
        self.write_internal(addr, true)
    }

    pub fn write(&mut self, addr: u32) -> CacheResponse {
        self.write_internal(addr, false)
    }

    /// Performs a write access to the cache according to the write policy.
    fn write_internal(&mut self, addr: u32, force: bool) -> CacheResponse {
        let (ppn, _page_offset) = bits::split_at(addr, self.global_config.pt.offset_size);
        let (block_addr, _block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];
        let (result, writeback, eviction) = match set.lookup(tag) {
            // Some block found: Hit
            Some(block) => {
                if self.config.write_policy == WriteBack {
                    block.borrow_mut().enfilthen();
                }
                (QueryResult::Hit, None, None)
            },
            // No block found: Miss
            None if self.config.write_miss_policy == NoWriteAllocate && !force => {
                (QueryResult::Miss, None, None)
            },
            None => {
                let new_entry = CacheEntry {
                    tag,
                    addr,
                    ppn,
                    dirty: true,
                };
                let evicted_block = set.push(new_entry);
                let evicted_addr = evicted_block.map(|b| b.addr);

                let writeback = evicted_block
                    .filter(|block| {
                        block.is_dirty()
                    })
                    .map(|block| block.addr);
                (QueryResult::Miss, writeback, evicted_addr)
            },
        };

        CacheResponse {
            tag,
            idx,
            result,
            writeback,
            eviction
        }
    }

    /// Invalidates all entries in teh cache that refer to the supplied PPN
    pub fn clean_ppn(&mut self, ppn: u32) -> Option<Vec<u32>> {
        let mut writebacks = Vec::<u32>::new();
        for set in self.sets.iter_mut() {
            if let Some(mut set_writebacks) = set.invalidate_entries_by_ppn(ppn) {
                writebacks.append(&mut set_writebacks);
            }
        }
        // FIXING THE CODE FOR SHIVAM: rustc 1.58 does not support this
        //(!writebacks.is_empty()).then_some(writebacks)

        if !writebacks.is_empty() {
            Some(writebacks)
        } else {
            None
        }
    }    
    
    pub fn clean_addr(&mut self, addr: u32) -> Option<Vec<u32>> {
        let (_ppn, _page_offset) = bits::split_at(addr, self.global_config.pt.offset_size);
        let (block_addr, _block_offset) = bits::split_at(addr, self.config.offset_size);
        let (_tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.sets[idx as usize];
        if let Some(writebacks) = set.invalidate_entries_by_addr(addr) {
            Some(writebacks)
        } else {
            None
        }
    }
}

/* === LRU Set === */

use std::{collections::VecDeque, cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
/// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
/// This one is tailored for use with the CPUCache. DO NOT GENERALIZE THIS AGAIN, PLEASE.
//
//  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
//  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
//  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
//  Also, the max set size is like, 16. Just a note for future reference.
struct LRUSet {
    inner: VecDeque<Rc<RefCell<CacheEntry>>>,
    capacity: usize,
}

impl LRUSet {
    fn new(capacity: usize) -> Self {
        let inner = VecDeque::<Rc<RefCell<CacheEntry>>>::with_capacity(capacity);
        LRUSet { inner, capacity }
    }

    /// Push an item to the LRU Set, potentially evicting the oldest item
    fn push(&mut self, entry: CacheEntry) -> Option<CacheEntry> {
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
    fn lookup(&mut self, tag: u32) -> Option<Rc<RefCell<CacheEntry>>> {
        let item_search = self.inner
            .iter()
            .position(|entry| entry.borrow().tag == tag );
        
        if let Some(item_idx) = item_search {
            let item = self.inner.remove(item_idx).unwrap();
            self.inner.push_front(item.clone());
            Some(item)
        } else {
            None
        }
    }

    /// Evicts any entry that corresponds to the supplied ppn. Returns a list of writebacks
    fn invalidate_entries_by_ppn(&mut self, ppn: u32) -> Option<Vec<u32>> {
        let mut writebacks = Vec::new();

        // Copy the entire LRU set without the invalid entries
        let new_inner = self.inner
            .iter()
            // filter out invalid entries and push them to writebacks if dirty
            .filter_map(|entry| {
                if entry.borrow().ppn == ppn {
                    if entry.borrow().is_dirty() {
                        writebacks.push(entry.borrow().addr);
                    }
                    None
                } else {
                    Some(entry.borrow().clone())
                }
            })
            // take raw entries and box them up for shipping
            .map(|raw_entry| {
                Rc::new(RefCell::new(raw_entry))
            })
            .collect();
        
        // Set the LRUSet's inner to be the filtered set
        // -- This drops the old inner completely.
        self.inner = new_inner;

        // FIXING THE CODE FOR SHIVAM: rustc 1.58 does not support this
        //(!writebacks.is_empty()).then_some(writebacks)

        // Return the writebacks
        if !writebacks.is_empty() {
            Some(writebacks)
        } else {
            None
        }
    }

    /// Evicts any entry that corresponds to the supplied ppn. Returns a list of writebacks
    fn invalidate_entries_by_addr(&mut self, addr: u32) -> Option<Vec<u32>> {
        let mut writebacks = Vec::new();

        // Copy the entire LRU set without the invalid entries
        let new_inner = self.inner
            .iter()
            // filter out invalid entries and push them to writebacks if dirty
            .filter_map(|entry| {
                if entry.borrow().addr == addr {
                    if entry.borrow().is_dirty() {
                        writebacks.push(entry.borrow().addr);
                    }
                    None
                } else {
                    Some(entry.borrow().clone())
                }
            })
            // take raw entries and box them up for shipping
            .map(|raw_entry| {
                Rc::new(RefCell::new(raw_entry))
            })
            .collect();
        
        // Set the LRUSet's inner to be the filtered set
        // -- This drops the old inner completely.
        self.inner = new_inner;

        // FIXING THE CODE FOR SHIVAM: rustc 1.58 does not support this
        //(!writebacks.is_empty()).then_some(writebacks)

        // Return the writebacks
        if !writebacks.is_empty() {
            Some(writebacks)
        } else {
            None
        }
    }
}

impl std::fmt::Debug for CPUCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "L{} Cache:", self.config.id)?;
        for (idx, set) in self.sets.iter().enumerate() {
            writeln!(f, "\tSet {:x}:", idx)?;
            for entry in set.inner.iter() {
                let e = entry.borrow();
                writeln!(f, "\t\taddr: {:x}\n\t\ttag: {:x}\n\t\tppn: {:x}\n\t\tdirty: {}",
                    e.addr, e.tag, e.ppn, if e.is_dirty() { "yes" } else { "no" })?;
            }
        }
        Ok(())
    }
}