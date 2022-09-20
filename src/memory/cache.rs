use crate::{config, utils::bits};
use super::Query;

pub struct CacheResponse {
    pub tag: u32,
    pub idx: u32,
    pub result: Query,
}

pub struct CacheEntry {
    tag: u32,
    age: u32,
}

pub struct Cache {
    data: Vec< // Set
            Vec< // Block
                Option<u32>>>, // Entry is either valid (Some) or invalid (None)
    config: config::CacheConfig,
    
}

impl Cache {
    pub fn new(config: config::CacheConfig) -> Self {
        let empty_set = vec![ None ; config.set_entries ];
        let cache_inner = vec![ empty_set ; config.sets ];
        Cache { 
            data: cache_inner,
            config,
        }
    }

    pub fn lookup(&mut self, addr: u32) -> CacheResponse {
        let (block_addr, block_offset) = bits::split_at(addr, self.config.offset_size);
        let (tag, idx) = bits::split_at(block_addr, self.config.idx_size);

        let set = &mut self.data[idx as usize];
        

        let searched_block = set.iter()
            .find(|&&block| block == Some(tag));

        let query_result = match searched_block {
            Some(valid_block) => {
                if valid_block.is_some() {
                    Query::Hit
                } else {
                    Query::Miss
                }
            },
            None => Query::Miss,
        };

        // TODO: This only supports 1-way associativity
        if query_result == Query::Miss {
            set[0] = Some(tag);
        }

        CacheResponse {
            tag,
            idx,
            result: query_result,
        }
    }
}