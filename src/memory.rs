mod page;
mod tlb;
mod cache;
use crate::{
    config::{self, Config, WritePolicy::*},
    memory::{
        page::PageTable,
        cache::CPUCache, 
        tlb::Tlb,
    }, utils::bits
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
        config: &Config
    ) -> Result<AccessEvent, Box<dyn std::error::Error>> {
        match config.address_type {
            config::AddressType::Virtual => {
                if addr > config.pt.max_virtual_addr {
                    error!("virtual address {:08x} is too large (maximum size is 0x{:x})",
                        addr, config.pt.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if addr > config.pt.max_physical_addr {
                    error!("physical address {:08x} is too large (maximum size is 0x{:x})",
                        addr, config.pt.max_physical_addr - 1);
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
#[derive(Debug)]
pub struct Memory {
    #[allow(dead_code)]
    tlb: Tlb,
    pt: PageTable,
    dc: CPUCache,
    l2: CPUCache,
    config: Config,
}

impl Memory {
    /// Configures all submodules of the memory system and initializes the memory simulation object.
    pub fn new(config: Config) -> Self {
        let tlb = Tlb::new(config.tlb.clone());
        let pt = PageTable::new(config.pt.clone());
        let dc = CPUCache::new(config.dc.clone(), config.clone());
        let l2 = CPUCache::new(config.l2.clone(), config.clone());
        Memory {tlb, pt, dc, l2, config}
    }

    /// Issue an access event to the memory system (which is either a read or a write).
    pub fn access(
        &mut self, 
        raw_access_type: char, 
        raw_addr: u32
    ) -> Result<AccessResult, Box<dyn std::error::Error>> {

        // Make sure addr is a reasonable size
        match self.config.address_type {
            config::AddressType::Virtual => {
                if raw_addr > self.config.pt.max_virtual_addr {
                    error!("virtual address {:08x} is too large (maximum size is 0x{:x})",
                        raw_addr, self.config.pt.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if raw_addr > self.config.pt.max_physical_addr {
                    error!("physical address {:08x} is too large (maximum size is 0x{:x})",
                        raw_addr, self.config.pt.max_physical_addr - 1);
                }
            }
        }

        let access_event = AccessEvent::from_raw(raw_access_type, raw_addr, &self.config)?;

        /* Step 1: Translate virtual address to physical address */

        let (physical_page_num, page_offset) = match self.config.address_type {
            // Convert virtual address to physical address as ppn and page offset
            config::AddressType::Virtual => {
                let (vpn, page_offset) = bits::split_at(access_event.addr(), self.config.pt.offset_size);

                if self.config.tlb.enabled {
                    //let (_tag, ppn) = self.tlb.lookup(vpn as usize);
                    //early return from block with translation
                    unimplemented!()
                }

                let pt_response = self.pt.translate(vpn);
                (pt_response.ppn, page_offset)
            },
            // Address is alreaady physical, just get the ppn and page offset
            config::AddressType::Physical => {
                bits::split_at(access_event.addr(), self.config.pt.offset_size)
            },
        };

        /* Step 2: Try to access data in caches in the order of DC -> L2 -> Memory */

        let dc_response = match access_event {
            AccessEvent::Read(addr) => self.dc.read(addr),
            AccessEvent::Write(addr) => self.dc.write(addr),
        };
        if let Some(writeback_addr) = dc_response.writeback {
            #[cfg(debug_assertions)]
            println!("writeback dc -> L2: {}", writeback_addr);
            self.l2.write(writeback_addr);
        }

        let l2_response: Option<cache::CacheResponse> = if self.config.l2.enabled {
            match dc_response.result {
                // If DC has a write through policy, then we write through to L2
                QueryResult::Hit if access_event.is_write() && self.config.dc.write_policy == WriteThrough => {
                    #[cfg(debug_assertions)]
                    println!("writing through dc -> L2: {}", access_event.addr());
                    Some(self.l2.write(access_event.addr()))
                },
                QueryResult::Miss => {
                    Some( match access_event {
                        AccessEvent::Read(addr) => self.l2.read(addr),
                        AccessEvent::Write(addr) => self.l2.write(addr),
                    })
                },
                _ => None,
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

    #[cfg(debug_assertions)]
    pub fn dbg_invalidate(&mut self, ppn: u32) {
        let writebacks = self.dc.clean_ppn(ppn);
    }

    #[cfg(debug_assertions)]
    pub fn dbg_print_state(&self, id: u32) {
        match id {
            1 => println!("{:?}", self.dc),
            2 => println!("{:?}", self.l2),
            _ => println!("Invalid state ID. Yeah, the interface is bad, who cares?")
        }
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

 /*
impl AccessResult { 
    /// Verifies that the memory simulation behavior is at least in accordance with the config.
    fn is_valid(&self, config: &Config) -> bool {
        todo!()
    }
}
*/

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