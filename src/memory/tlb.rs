use crate::config;

#[derive(Debug, Copy, Clone)]
pub struct TLBEntry {
    tag: u32,
    addr: u32,
    ppn: u32,
    dirty: bool,
}

impl TLBEntry {
    fn enfilthen(&mut self) {
        self.dirty = true;
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

pub struct TLB {
    sets: Vec<LRUSet>,
    config: config::TLBConfig,
}

impl TLB {
    pub fn new(config: config::TLBConfig) -> Self {
        let empty_set = LRUSet::new(config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        TLB { sets, config, }
    }
}

/* === LRU Set === */

use std::{collections::VecDeque, cell::RefCell, rc::Rc};

/// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
/// This one is tailored for use with the CPUCache. 
// 
// DO NOT GENERALIZE THIS AGAIN, PLEASE.
//
//  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
//  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
//  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
//  Also, the max set size is like, 16. Just a note for future reference.
#[derive(Clone, Debug)]
struct LRUSet {
    inner: VecDeque<Rc<RefCell<TLBEntry>>>,
    capacity: usize,
}

impl LRUSet {
    fn new(capacity: usize) -> Self {
        let inner = VecDeque::<Rc<RefCell<TLBEntry>>>::with_capacity(capacity);
        LRUSet { inner, capacity }
    }

    /// Push an item to the LRU Set, potentially evicting the oldest item
    fn push(&mut self, entry: TLBEntry) -> Option<TLBEntry> {
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
    fn lookup(&mut self, tag: u32) -> Option<Rc<RefCell<TLBEntry>>> {
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
    fn invalidate_entries(&mut self, ppn: u32) -> Option<Vec<u32>> {
        let mut writebacks = Vec::new();

        // Copy the entires LRU set (I know) removing the invalid entries
        let new_inner = self.inner
            .iter()
            // filter out invalid entries and push them to writebacks if dirty
            .filter_map(|entry| {
                if entry.borrow().ppn == ppn {
                    #[cfg(debug_assertions)]
                    println!("entry {} in TLB is invalidated due to ppn {} being evicted", entry.borrow().addr, ppn);
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
        self.inner = new_inner;

        // Return the writebacks
        (!writebacks.is_empty()).then_some(writebacks)
    }
}

impl std::fmt::Debug for TLB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TLB:")?;
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