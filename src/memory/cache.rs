use std::collections::VecDeque;
use crate::{config, utils::bits};
use super::{Query, lru::Set};

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: Query,
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

    pub fn lookup(&mut self, addr: u32) -> CacheResponse {
        let (block_addr, block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.data[idx as usize];

        let block = set.lookup(tag);

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