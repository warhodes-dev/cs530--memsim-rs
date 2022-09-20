mod page;
mod tlb;
mod cache;
use crate::{
    trace,
    config::{Config, AddressType},
    utils::{self, bits},
    memory::{
        page::{PageTable, PhysicalAddr, VirtualAddr},
        cache::Cache, 
        tlb::Tlb,
    }
};

pub struct Memory {
    tlb: Tlb,
    pt: PageTable,
    dc: Cache,
    l2: Cache,
    address_type: AddressType,
}

impl Memory {
    pub fn new(config: Config) -> Self {
        let tlb = Tlb::new(&config.tlb);
        let pt = PageTable::new(config.pt);
        let dc = Cache::new(config.dc);
        let l2 = Cache::new(config.l2);
        let address_type = config.address_type;
        Memory {tlb, pt, dc, l2, address_type}
    }

    pub fn access(&mut self, raw_trace: trace::RawTrace) -> Result<AccessEvent, Box<dyn std::error::Error>> {
        let raw_addr = raw_trace.addr();

        // Make sure addr is a reasonable size
        match self.address_type {
            AddressType::Virtual => {

            },
            AddressType::Physical => {

            }
        }


        let (physical_page_num, page_offset) = match self.address_type {
            AddressType::Virtual => {
                //let (vpn, offset) = bits::split_at(raw_addr, self.config.pt.offset_size);
                //let (_tag, ppn) = self.tlb.lookup(vpn as usize);
                unimplemented!()
            },
            AddressType::Physical => {
                let phys_addr = self.pt.passthrough(raw_addr);
                (phys_addr.page_num, phys_addr.page_offset)
            },
        };

        let dc_response = self.dc.lookup(raw_addr);

        let event = AccessEvent {
            addr: raw_addr,
            page_offset,
            physical_page_num,
            dc_tag: dc_response.tag,
            dc_idx: dc_response.idx,
            dc_res: Some(dc_response.result),
            ..Default::default()
        };

        Ok(event)
    }
}

/// Represents the details of a successful access of the memory simulation.
#[derive(Default)]
pub struct AccessEvent {
    addr: u32,
    virtual_page_num: Option<u32>,
    page_offset: u32,
    tlb_tag: Option<u32>,
    tlb_idx: Option<u32>,
    tlb_res: Option<Query>,
    page_table_res: Option<Query>,
    physical_page_num: u32,

    dc_tag: u32,
    dc_idx: u32,
    dc_res: Option<Query>,
    l2_tag: Option<u32>,
    l2_idx: Option<u32>,
    l2_res: Option<Query>,
}

impl AccessEvent { 
    fn is_valid(&self, config: &Config) -> bool {
        // TODO: make this into actuality
        true
    }
}

impl std::fmt::Display for AccessEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, 
            //addr  pg # pgoff tbtg tbix tlbr ptrs phypg dctag dcidx dcrs l2tg l2ix l2rs
            "{:08x} {:6} {:4x} {:6} {:3} {:4} {:4} {:4x} {:6x} {:3x} {:4} {:6} {:3} {:4}",
            self.addr,
            self.virtual_page_num.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.page_offset,
            self.tlb_tag.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.tlb_idx.map_or("".to_string(), |n| format!("{:3x}", n)),
            self.tlb_res.as_ref().map_or("", |q| q.as_str()),
            self.page_table_res.as_ref().map_or("", |q| q.as_str()),
            self.physical_page_num,
            self.dc_tag,
            self.dc_idx,
            self.dc_res.as_ref().map_or("", |q| q.as_str()),
            self.l2_tag.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.l2_idx.map_or("".to_string(), |n| format!("{:3x}", n)),
            self.l2_res.as_ref().map_or("", |q| q.as_str()),
        )
    }
}

#[derive(PartialEq, Eq)]
pub enum Query {
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
    fn as_str(&self) -> &'static str {
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