mod page;
mod tlb;
mod cache;
use crate::{
    trace,
    config,
    memory::{
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
        let pt = PageTable::new(config.pt);
        let dc = Cache::new(config.dc);
        let l2 = Cache::new(config.l2);
        Memory {tlb, pt, dc, l2}
    }
    pub fn access(&self, raw_trace: trace::RawTrace) -> AccessEvent {
        let addr = raw_trace.addr();
        unimplemented!()
    }
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
        write!(f, 
            "{:08x} {:6x} {:4x} {:6x} {:3x} {:4} {:4} {:4x} {:6x} {:3x} {:4} {:6x} {:3x} {:4}",
            self.virtual_addr,
            self.virtual_page,
            self.page_offset,
            self.tlb_tag,
            self.tlb_idx,
            if let Some(q) = &self.tlb_res { q.as_str() } else { "" },
            if let Some(q) = &self.page_table_res { q.as_str() } else { "" },
            self.phys_page,
            self.dc_tag,
            self.dc_idx,
            if let Some(q) = &self.dc_res { q.as_str() } else { "" },
            self.l2_tag,
            self.l2_idx,
            if let Some(q) = &self.l2_res { q.as_str() } else { "" },
        )
    }
}

enum Query {
    Hit,
    Miss,
}

impl Query {
    fn as_str(&self) -> &str {
        match self {
            Query::Hit => "hit",
            Query::Miss => "miss",
        }
    }
}