#[allow(unused_imports)]
use crate::{
    config,
    utils::bits,
};

pub struct PageTableResponse {
    pub vpn: u32,
    pub ppn: u32,
    pub evicted_entry: Option<u32>,
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableEntry {
    vpn: u32,
    ppn: u32,
}

pub struct PageTable {
    entries: LRUSet,
    config: config::PageTableConfig,
}

impl PageTable {
    pub fn new(config: config::PageTableConfig) -> Self {
        let entries = LRUSet::new(config.physical_pages as usize);
        PageTable {
            entries,
            config,
        }
    }

    /// Translates a virtual page number to a physical page number. 
    /// Can fault and cause pages to be allocated/evicted.
    pub fn translate(&mut self, vpn: u32) -> PageTableResponse {

        match self.entries.lookup(vpn) {
            Some(pte) => {
                
            },
            None => {

            }
        }

        unimplemented!()
    }
}

/* === LRU Set === */

use std::{collections::VecDeque, cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
/// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
/// This one is tailored for use with the PageTable. DO NOT GENERALIZE THIS AGAIN, PLEASE.
//
//  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
//  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
//  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
//  Also, the max set size is like, 16. Just a note for future reference.
struct LRUSet {
    inner: VecDeque<PageTableEntry>,
    capacity: usize,
}

impl LRUSet {
    fn new(capacity: usize) -> Self {
        let inner = VecDeque::<PageTableEntry>::with_capacity(capacity);
        LRUSet { inner, capacity }
    }

    /// Push an item to the LRU Set, potentially evicting the oldest item
    fn push(&mut self, entry: PageTableEntry) -> Option<PageTableEntry> {
        let evicted_item = if self.inner.len() >= self.capacity {
            self.inner.pop_back()
                .map(|entry| {
                    entry.clone()
                })
        } else { None };
        self.inner.push_front(entry);
        evicted_item
    }

    /// Look up an item in the LRU Set. If found, the item is 'touched' and moved to the front
    /// of the queue.
    fn lookup(&mut self, vpn: u32) -> Option<PageTableEntry> {
        let item_search = self.inner
            .iter()
            .position(|entry| entry.vpn == vpn);
        
        if let Some(item_idx) = item_search {
            let item = self.inner.remove(item_idx).unwrap();
            self.inner.push_front(item.clone());
            Some(item)
        } else {
            None
        }
    }
}

impl std::fmt::Debug for PageTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Page Table:")?;
        for pte in self.entries.inner.iter() {
            writeln!(f, "\tvpn: {} -> ppn: {}", pte.vpn, pte.ppn)?;
        }
        Ok(())
    }
}