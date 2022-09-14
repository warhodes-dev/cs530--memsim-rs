use crate::config;

pub struct PageTable {

}

impl PageTable {
    pub fn new(config: config::PageTableConfig) -> Self {
        unimplemented!()
    }
}

struct VirtualAddr {
    page_num: u32,
    page_offset: u32,
}

// 1100 | 1000 0010
//    