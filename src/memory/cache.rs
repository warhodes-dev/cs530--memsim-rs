use crate::config;

pub type CacheEntry = (u32, u32);

pub struct Cache(
    Vec< // Sets
        Vec< // Set Entries
            Option<CacheEntry>>>); // Entry is either valid (Some) or invalid (None)

impl Cache {
    pub fn new(cfg: &config::CacheConfig) -> Self {
        let empty_set = vec![ None ; cfg.set_entries ];
        let cache_inner = vec![ empty_set ; cfg.sets ];
        Cache(cache_inner)
    }

    pub fn lookup(&self, i: usize) {
        
    }
}