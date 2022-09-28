mod page;
mod tlb;
mod cache;
use crate::{
    trace,
    config::{self, Config, MemoryConfig},
    utils::{self, bits},
    memory::{
        page::{PageTable, PhysicalAddr, VirtualAddr},
        cache::CPUCache, 
        tlb::Tlb,
    }
};

/// Represents the input access events.
/// 
/// Contains either a physical or virtual address (based on config) and can
/// either be a AccessEvent::Read or a AccessEvent::Write.
type AccessEvent = trace::TraceEvent;

impl AccessEvent {
    fn is_write(&self) -> bool {
        match self {
            AccessEvent::Write(_) => true,
            _ => false,
        }
    }
}

/// The simulated memory system
pub struct Memory {
    tlb: Tlb,
    pt: PageTable,
    dc: CPUCache,
    l2: CPUCache,
    config: MemoryConfig,
}

impl Memory {
    /// Configures all submodules of the memory system and initializes the memory simulation object.
    pub fn new(config: Config) -> Self {
        let tlb = Tlb::new(&config.tlb);
        let pt = PageTable::new(config.pt);
        let dc = CPUCache::new(config.dc);
        let l2 = CPUCache::new(config.l2);
        let config = config.mem;
        Memory {tlb, pt, dc, l2, config}
    }

    /// Issue an access event to the memory system (which is either a read or a write).
    pub fn access(&mut self, request: AccessEvent) -> Result<AccessResult, Box<dyn std::error::Error>> {
        let raw_addr = request.addr();

        // Make sure addr is a reasonable size
        match self.config.address_type {
            config::AddressType::Virtual => {
                if raw_addr > self.config.max_virtual_addr {
                    panic!("virtual address {:08x} is too large (maximum size is 0x{:x})",
                        raw_addr, self.config.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if raw_addr > self.config.max_physical_addr {
                    panic!("physical address {:08x} is too large (maximum size is 0x{:x})",
                        raw_addr, self.config.max_physical_addr - 1);
                }
            }
        }

        let (physical_page_num, page_offset) = match self.config.address_type {
            config::AddressType::Virtual => {
                //let (vpn, offset) = bits::split_at(raw_addr, self.config.pt.offset_size);
                //let (_tag, ppn) = self.tlb.lookup(vpn as usize);
                unimplemented!()
            },
            config::AddressType::Physical => {
                let phys_addr = self.pt.passthrough(raw_addr);
                (phys_addr.page_num, phys_addr.page_offset)
            },
        };

        let dc_response = self.dc.lookup(request);

        let event = AccessResult {
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

/// Details the interior behavior of a simulated access to the memory system.
#[derive(Default)]
pub struct AccessResult {
    addr: u32,
    virtual_page_num: Option<u32>,
    page_offset: u32,
    tlb_tag: Option<u32>,
    tlb_idx: Option<u32>,
    tlb_res: Option<QueryResult>,
    page_table_res: Option<QueryResult>,
    physical_page_num: u32,

    dc_tag: u32,
    dc_idx: u32,
    dc_res: Option<QueryResult>,
    l2_tag: Option<u32>,
    l2_idx: Option<u32>,
    l2_res: Option<QueryResult>,
}

impl AccessResult { 
    /// Verifies that the memory simulation behavior is at least in accordance with the config.
    fn is_valid(&self, config: &Config) -> bool {
        todo!()
    }
}

impl std::fmt::Display for AccessResult {
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

#[derive(Eq, PartialEq)]
/// A query to any of the cache subsystems, which can either be QueryResult::Hit
pub enum QueryResult {
    Hit,
    Miss,
}

impl QueryResult {
    #[allow(dead_code)]
    fn to_string(&self) -> String {
        String::from(match self {
            QueryResult::Hit => "hit",
            QueryResult::Miss => "miss",
        })
    }
    fn as_str(&self) -> &'static str {
        match self {
            QueryResult::Hit => "hit",
            QueryResult::Miss => "miss",
        }
    }
}