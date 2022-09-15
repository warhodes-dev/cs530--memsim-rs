use crate::config;

pub struct Cache {
    cfg: config::CacheConfig,
}

impl Cache {
    pub fn new(cfg: config::CacheConfig) -> Self {
        Cache {
            cfg,
        }
    }
}