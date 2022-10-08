use std::{collections::VecDeque, cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
/// A simple LRU Set which evicts elements upon insertion such that the set never exceeds `capacity`
//
//  This could be faster. By using a VecDequeue, 'touching' an item of the cache is O(n). Ideally,
//  some kind of linked hash map could be used for O(1) lookup _and_ O(1) touch. I suspect that
//  designing that from scratch would have some serious rust shenanigans that I don't want to deal with.
//  Also, the max set size is like, 16. Just a note for future reference.
pub struct LRUSet<T> {
    inner: VecDeque<Rc<RefCell<T>>>,
    capacity: usize,
    search: fn(u32, T) -> bool,
}

impl<T> LRUSet<T>
where T: Copy + Clone {
    pub fn new(search: fn(u32, T) -> bool, capacity: usize) -> Self {
        let inner = VecDeque::<Rc<RefCell<T>>>::with_capacity(capacity);
        LRUSet { inner, capacity, search }
    }

    /// Push an item to the LRU Set, potentially evicting the oldest item
    pub fn push(&mut self, entry: T) -> Option<T> {
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
    pub fn lookup(&mut self, tag: u32) -> Option<Rc<RefCell<T>>> {
        let item_search = self.inner
            .iter()
            .position(|entry| (self.search)(tag, *entry.borrow()) );
        
        if let Some(item_idx) = item_search {
            let item = self.inner.remove(item_idx).unwrap();
            self.inner.push_front(item.clone());
            Some(item)
        } else {
            None
        }
    }
}