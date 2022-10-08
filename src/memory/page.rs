#[allow(unused_imports)]
use crate::{
    config,
    utils::bits,
};

#[derive(Debug)]
pub struct PageTable {
    config: config::PageTableConfig,
}

impl PageTable {
    pub fn new(config: config::PageTableConfig) -> Self {
        PageTable {
            config,
        }
    }

    /// Destructures a raw address without translating from virtual to physical
    pub fn passthrough(&self, addr: u32) -> PhysicalAddr {
        // Do not use this function if page table is enabled
        assert!(!self.config.enabled);
        let (page_num, page_offset) = bits::split_at(addr, self.config.offset_size);
        PhysicalAddr { page_num, page_offset }
    }
}

pub struct PhysicalAddr {
    pub page_num: u32,
    pub page_offset: u32,
}

#[allow(dead_code)]
pub struct VirtualAddr {
    pub page_num: u32,
    pub page_offset: u32,
}

/* === LRU Set === 

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

    /// Evicts any entry that corresponds to the supplied ppn
    fn invalidate_entries(&mut self, ppn: u32) {

    }
}
*/