#[allow(unused_imports)]
use crate::{
    config,
    utils::bits,
    memory::QueryResult,
};

pub struct PageTableResponse {
    pub vpn: u32,
    pub ppn: u32,
    pub page_offset: u32,
    pub res: QueryResult,
    pub evicted_ppn: Option<u32>,
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableEntry {
    vpn: u32,
    ppn: u32,
}

pub struct PageTable {
    entries: LRUTable,
    config: config::PageTableConfig,
}

impl PageTable {
    pub fn new(config: config::PageTableConfig) -> Self {
        let entries = LRUTable::new(config.physical_pages as usize);
        PageTable { entries, config, }
    }

    /// Translates a virtual page number to a physical page number. 
    /// Can fault and cause pages to be allocated/evicted.
    pub fn translate(&mut self, addr: u32) -> PageTableResponse {
        let (vpn, page_offset) = bits::split_at(addr, self.config.offset_size);

        match self.entries.lookup(vpn) {
            Some(ppn) => {
                let res = QueryResult::Hit;
                PageTableResponse { 
                    vpn: vpn,
                    ppn,
                    page_offset,
                    res: res, 
                    evicted_ppn: None 
                }
            },
            // Page fault: No page was found, so we must insert one (and optionally evict one)
            None => {
                let res = QueryResult::Miss;
                let (ppn, evicted_ppn) = self.entries.push(vpn);
                PageTableResponse { 
                    vpn: vpn,
                    ppn,
                    page_offset,
                    res: res,
                    evicted_ppn 
                }
            }
        }
    }

    

    /* Simply converts an addr into a ppn and offset based on config (no translation)
    pub fn passthrough(&self, addr: u32) -> PageTableResponse {
        let (ppn, page_offset) = bits::split_at(addr, self.config.offset_size);
        PageTableResponse {
            vpn: None,
            ppn,
            page_offset,
            res: None,
            evicted_ppn: None,
        }
    }
    */
}

/* === LRU Set === */

use std::collections::VecDeque;
#[derive(Clone, Debug)]
/// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
/// This one is tailored for use with the PageTable. DO NOT GENERALIZE THIS AGAIN, PLEASE.
//
//  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
//  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
//  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
//  Also, the max set size is like, 16. Just a note for future reference.
struct LRUTable {
    inner: VecDeque<PageTableEntry>,
    capacity: usize,
}

impl LRUTable {
    fn new(capacity: usize) -> Self {
        let inner = VecDeque::<PageTableEntry>::with_capacity(capacity);
        LRUTable { inner, capacity }
    }

    /// Push an item to the LRU Table, potentially evicting the oldest item
    fn push(&mut self, vpn: u32) -> (u32, Option<u32>) {
        // If table is full, evict an item
        let (ppn, evicted_ppn) = if self.inner.len() >= self.capacity {
            let evicted_ppn = self.inner.pop_back()
                .map(|entry| {
                    entry.clone()
                })
                .expect("Failed to pop_back of deque, for some reason")
                .ppn;
            
            let ppn = evicted_ppn;
            (ppn, Some(evicted_ppn))
        // Otherwise, allocate a new item
        } else { 
            let ppn = self.inner.len() as u32;
            let evicted_ppn = None;
            (ppn, evicted_ppn)
        };

        let entry = PageTableEntry{ vpn, ppn };
        self.inner.push_front(entry);

        (ppn, evicted_ppn)
    }

    /// Look up an item in the LRU Set. If found, the item is 'touched' and moved to the front
    /// of the queue.
    fn lookup(&mut self, vpn: u32) -> Option<u32> {
        let item_search = self.inner
            .iter()
            .position(|entry| entry.vpn == vpn);
        
        if let Some(item_idx) = item_search {
            let item = self.inner.remove(item_idx).unwrap();
            self.inner.push_front(item.clone());
            Some(item.ppn)
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