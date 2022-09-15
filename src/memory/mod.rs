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

        if self.pt.cfg.enabled {

        } else {

        }
        
        AccessEvent {
            addr,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct AccessEvent {
    addr: u32,
    virtual_page: Option<u32>,
    page_offset: u32,
    tlb_tag: Option<u32>,
    tlb_idx: Option<u32>,
    tlb_res: Option<Query>,
    page_table_res: Option<Query>,
    phys_page: u32,

    dc_tag: u32,
    dc_idx: u32,
    dc_res: Query,
    l2_tag: Option<u32>,
    l2_idx: Option<u32>,
    l2_res: Option<Query>,
}

impl std::fmt::Display for AccessEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, 
            //addr  pg # pgoff tbtg tbix tlbr ptrs phypg dctag dcidx dcrs l2tg l2ix l2rs
            "{:08x} {:6} {:4x} {:6} {:3} {:4} {:4} {:4x} {:6x} {:3x} {:4} {:6} {:3} {:4}",
            self.addr,
            self.virtual_page.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.page_offset,
            self.tlb_tag.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.tlb_idx.map_or("".to_string(), |n| format!("{:3x}", n)),
            self.tlb_res.as_ref().map_or("", |q| q.as_str()),
            self.page_table_res.as_ref().map_or("", |q| q.as_str()),
            self.phys_page,
            self.dc_tag,
            self.dc_idx,
            self.dc_res.as_str(),
            self.l2_tag.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.l2_idx.map_or("".to_string(), |n| format!("{:3x}", n)),
            self.l2_res.as_ref().map_or("", |q| q.as_str()),
        )
    }
}

enum Query {
    Hit,
    Miss,
}

impl Query {
    #[allow(dead_code)]
    fn to_string(&self) -> String {
        String::from(match self {
            Query::Hit => "hit",
            Query::Miss => "miss",
        })
    }
    fn as_str(&self) -> &str {
        match self {
            Query::Hit => "hit",
            Query::Miss => "miss",
        }
    }
}

// TODO: Remove this
impl Default for Query {
    fn default() -> Self {
        Query::Miss
    }
}

/* 
#[cfg(test)]
mod test {
    use super::AccessEvent;

    #[test]
    fn test_output_string() {
        let ae = AccessEvent {
            addr: 0xc83,
            virtual_page: Some(0xc),
            page_offset: 0x83,
            ..Default::default()
        };
        println!("{ae}")
    }
}
*/