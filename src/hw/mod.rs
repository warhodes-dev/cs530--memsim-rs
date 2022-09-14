mod page;
mod tlb;
mod cache;
use crate::{
    trace,
    config,
    hw::{
        page::PageTable, 
        cache::Cache, 
        tlb::Tlb,
    }
};

pub struct Memory {
    tlb: Tlb,
    pt: PageTable,
    dc: Cache,
    l2: Cache,
}

impl Memory {
    pub fn new(config: config::Config) -> Self {
        let tlb = Tlb::new(config.tlb);
        println!("TLB: {:?}", tlb);
        let pt = PageTable::new(config.pt);
        let dc = Cache::new(config.dc);
        let l2 = Cache::new(config.l2);
        Memory {tlb, pt, dc, l2}
    }
    pub fn access(&self, raw_trace: trace::RawTrace) -> AccessEvent {
        AccessEvent::default()
    }
}

enum Query {
    Hit,
    Miss,
}

#[derive(Default)]
pub struct AccessEvent {
    virtual_addr: u32,
    virtual_page: u32,
    page_offset: u32,
    tlb_tag: u32,
    tlb_idx: u32,
    tlb_res: Option<Query>,
    page_table_res: Option<Query>,
    phys_page: u32,

    dc_tag: u32,
    dc_idx: u32,
    dc_res: Option<Query>,
    l2_tag: u32,
    l2_idx: u32,
    l2_res: Option<Query>,
}

impl std::fmt::Display for AccessEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}