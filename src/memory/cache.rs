use std::collections::VecDeque;
use crate::{trace, config, utils::bits};
use super::{Query, lru::Set};

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: Query,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CacheEntry {
    Clean(u32),
    Dirty(u32),
}

pub struct Cache {
    data: Vec<Set<u32>>,
    config: config::CacheConfig,
}

impl Cache {
    pub fn new(config: config::CacheConfig) -> Self {
        let empty_set = Set::new(config.set_entries as usize);
        let sets = vec![ empty_set ; config.sets as usize ];
        Cache { 
            data: sets,
            config,
        }
    }

    pub fn lookup(&mut self, request: trace::RawTrace) -> CacheResponse {
        let addr = request.addr();
        let (block_addr, block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.data[idx as usize];

        let block = set.lookup(tag);

        // This is all fucked. Cache structure needs to be changed to handle Clean/Dirty and
        // program logic needs to handle read/write
        let query_result = if block.is_some() {
            Query::Hit
        } else {
            set.push(tag);
            Query::Miss
        };

        CacheResponse {
            tag,
            idx,
            result: query_result,
        }
    }
}

impl CacheEntry {
    fn inner(&self) -> u32 {
        match self {
            CacheEntry::Clean(n) => *n,
            CacheEntry::Dirty(n) => *n,
        }
    }
}