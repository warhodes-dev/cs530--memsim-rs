use crate::config;

pub struct PageTable {
}

impl PageTable {
    pub fn new(cfg: &config::PageTableConfig) -> Self {
        PageTable {
        }
    }
}

struct VirtualAddr {
    page_num: u32,
    page_offset: u32,
}

struct PhysicalAddr {
    page_num: u32,
    page_offset: u32,
}

// 1100 | 1000 0010
// | ||
//  Y  L___ TLB index
//  L___ TLB tag