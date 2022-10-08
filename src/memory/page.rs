#[allow(unused_imports)]
use crate::{
    config,
    utils::bits,
    lru::LRUSet,
};


pub struct PageTable {
    config: config::PageTableConfig,
}

impl PageTable {
    pub fn new(config: config::PageTableConfig) -> Self {
        PageTable {
            config,
        }
    }

    /// Destructures a raw address without translating from virtual to physical
    pub fn passthrough(&self, addr: u32) -> PhysicalAddr {
        // Do not use this function if page table is enabled
        assert!(!self.config.enabled);
        let (page_num, page_offset) = bits::split_at(addr, self.config.offset_size);
        PhysicalAddr { page_num, page_offset }
    }
}

pub struct PhysicalAddr {
    pub page_num: u32,
    pub page_offset: u32,
}

#[allow(dead_code)]
pub struct VirtualAddr {
    pub page_num: u32,
    pub page_offset: u32,
}

// 1100 | 1000 0010
// | ||
//  Y  L___ TLB index
//  L___ TLB tag