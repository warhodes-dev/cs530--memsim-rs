mod page;
mod tlb;
mod cache;
use crate::{
    config::{self, Config, MemoryConfig, WriteMissPolicy::*, WritePolicy::*},
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
/// either be a `Read` or a `Write`.
pub enum AccessEvent {
    Read(u32),
    Write(u32),
}

impl AccessEvent {
    fn from_raw(
        access_type: char, 
        addr: u32, 
        config: &MemoryConfig
    ) -> Result<AccessEvent, Box<dyn std::error::Error>> {
        match config.address_type {
            config::AddressType::Virtual => {
                if addr > config.max_virtual_addr {
                    error!("virtual address {:08x} is too large (maximum size is 0x{:x})",
                        addr, config.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if addr > config.max_physical_addr {
                    error!("physical address {:08x} is too large (maximum size is 0x{:x})",
                        addr, config.max_physical_addr - 1);
                }
            }
        }

        let access_event = match access_type {
            'r' | 'R' => AccessEvent::Read(addr),
            'w' | 'W' => AccessEvent::Write(addr),
            _ => error!("invalid access type: {}", access_type)
        };
        Ok(access_event)
    }

    fn addr(&self) -> u32 {
        match self {
            AccessEvent::Write(addr) => *addr,
            AccessEvent::Read(addr) => *addr,
        }
    }

    fn is_write(&self) -> bool {
        match self {
            AccessEvent::Write(_) => true,
            AccessEvent::Read(_) => false,
        }
    }
}

/// The simulated memory system.
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
    pub fn access(
        &mut self, 
        raw_access_type: char, 
        raw_addr: u32
    ) -> Result<AccessResult, Box<dyn std::error::Error>> {
        //translate to phys first...
        // check tlb
        // by the way, study performance equations
        let access_event = AccessEvent::from_raw(raw_access_type, raw_addr, &self.config)?;

        // Make sure addr is a reasonable size
        match self.config.address_type {
            config::AddressType::Virtual => {
                if raw_addr > self.config.max_virtual_addr {
                    error!("virtual address {:08x} is too large (maximum size is 0x{:x})",
                        raw_addr, self.config.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if raw_addr > self.config.max_physical_addr {
                    error!("physical address {:08x} is too large (maximum size is 0x{:x})",
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
                let phys_addr = self.pt.passthrough(raw_addr); // TODO: I don't like this
                (phys_addr.page_num, phys_addr.page_offset)
            },
        };

        let dc_response = self.dc.lookup(&access_event);
        if let Some(writeback_addr) = dc_response.writeback {
            let writeback_event = AccessEvent::Write(writeback_addr);
            self.l2.lookup(&writeback_event);
        }

        let l2_response: Option<cache::CacheResponse> = if self.l2.config.enabled {
            match dc_response.result {
                QueryResult::Hit => {
                    // If DC has a write through policy, then we write through to L2
                    if access_event.is_write() && self.dc.config.write_policy == WriteThrough {
                        Some(self.l2.lookup(&access_event))
                    } else {
                        None
                    }
                },
                QueryResult::Miss => {
                    Some(self.l2.lookup(&access_event))
                },
            }
        } else {
            None
        };

        let event = AccessResult {
            addr: raw_addr,
            page_offset,
            physical_page_num,
            dc_tag: dc_response.tag,
            dc_idx: dc_response.idx,
            dc_res: Some(dc_response.result),
            l2_tag: l2_response.as_ref().map(|r| r.tag),
            l2_idx: l2_response.as_ref().map(|r| r.idx),
            l2_res: l2_response.as_ref().map(|r| r.result),
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
/// A query to any of the cache subsystems, which can either be `Hit` or `Miss`
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