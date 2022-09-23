use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Set<T>
where T: Copy + Clone {
    pub inner: VecDeque<T>,
    capacity: usize,
}

impl<T> Set<T> 
where 
    T: Copy + Clone,
    T: PartialEq + Eq,
{
    pub fn new(capacity: usize) -> Self {
        let inner = VecDeque::<T>::with_capacity(capacity);
        Set { inner, capacity }
    }

    pub fn push(&mut self, entry: T) {
        if self.inner.len() >= self.capacity {
            self.inner.pop_back();
        }
        self.inner.push_front(entry);
    }

    pub fn lookup(&mut self, entry: T) -> Option<T> {
        self.inner
            .iter()
            .position(|&i| i == entry)
            .map(|item_idx| {
                let item = self.inner.remove(item_idx).unwrap();
                self.inner.push_front(item);
                item
        })
    }
}