mod page;
mod tlb;
mod cache;

use crate::{
    config::{self, Config, WritePolicy::*},
    memory::{
        page::{PageTable, PageTableResponse},
        cache::CPUCache, 
        tlb::{TLB,TLBResponse},
    }, utils::bits
};

struct TranslationResponse {
    vpn: Option<u32>,
    ppn: u32,
    page_offset: u32,
    pt_response: Option<PageTableResponse>,
    tlb_response: Option<TLBResponse>,
}

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
    ) -> Result<AccessEvent, Box<dyn std::error::Error>> {
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
    tlb: TLB,
    pt: PageTable,
    dc: CPUCache,
    l2: CPUCache,
    config: Config,
}

impl Memory {
    /// Configures all submodules of the memory system and initializes the memory simulation object.
    pub fn new(config: Config) -> Self {
        let tlb = TLB::new(config.tlb.clone());
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
    ) -> Result<MemoryResponse, Box<dyn std::error::Error>> {

        // Make sure addr is a reasonable size
        match self.config.address_type {
            config::AddressType::Virtual => {
                if raw_addr > self.config.pt.max_virtual_addr - 1 {
                    error!("virtual address {0:08x}({0}) is too large (maximum size is {0:08x}({1}))",
                        raw_addr, self.config.pt.max_virtual_addr - 1);
                }
            },
            config::AddressType::Physical => {
                if raw_addr > self.config.pt.max_physical_addr - 1 {
                    error!("physical address {0:08x}({0}) is too large (maximum size is {0:08x}({1}))",
                        raw_addr, self.config.pt.max_physical_addr - 1);
                }
            }
        }

        /* Step 1: Translate virtual address to physical address */

        let translation_response = match self.config.address_type {
            config::AddressType::Physical => {
                let (ppn, page_offset) = bits::split_at(raw_addr, self.config.pt.offset_size);
                TranslationResponse {
                    ppn,
                    page_offset,
                    vpn: None,
                    pt_response: None,
                    tlb_response: None,
                }
            },
            config::AddressType::Virtual => {
                let ppn;
                let vpn;
                let page_offset;

                // FIXING THE CODE FOR SHIVAM: rustc 1.58 does not support this
                //let optional_tlb_response = self.config.tlb.enabled.then_some(self.tlb.lookup(raw_addr));

                let optional_tlb_response = if self.config.tlb.enabled {
                    Some(self.tlb.lookup(raw_addr))
                } else {
                    None
                };

                let optional_pt_response = match optional_tlb_response {
                    // TLB Disabled: go to page table
                    None => Some(self.pt.translate(raw_addr)),
                    // TLB Miss: go to page table and then update tlb
                    Some(ref tlb_response) if tlb_response.result == QueryResult::Miss => {
                        let pt_response = self.pt.translate(raw_addr);

                        if let Some(evicted_ppn) = pt_response.evicted_ppn {
                            // must invalidated tlb entries before inserting a new entry
                            self.tlb.clean_ppn(evicted_ppn);
                        }
                        self.tlb.push(pt_response.vpn, pt_response.ppn);

                        Some(pt_response)
                    }
                    // TLB hit: No need to access page table
                    Some(_/* TLB hit */) => { self.pt.translate(raw_addr); None}
                };

                // Invalidate entries in L2, DC, TLB, if a PTE was evicted
                if let Some(evicted_ppn) = optional_pt_response.as_ref().map(|ptr| ptr.evicted_ppn).flatten() {
                    self.dc.clean_ppn(evicted_ppn);
                    self.l2.clean_ppn(evicted_ppn);
                }

                // Get ppn, vpn, and page_offset for reporting 
                if let Some(tlb_response) = optional_tlb_response.as_ref().filter(|t| t.result == QueryResult::Hit) {
                    ppn = tlb_response.ppn.unwrap();
                    vpn = tlb_response.vpn;
                    page_offset = tlb_response.page_offset;
                } else if let Some(ref pt_response) = optional_pt_response {
                    ppn = pt_response.ppn;
                    vpn = pt_response.vpn;
                    page_offset = pt_response.page_offset;
                } else {
                    panic!("Somehow, neither the page table nor the tlb produced any ppn, vpn, or page_offset")
                }

                TranslationResponse {
                    ppn,
                    page_offset,
                    vpn: Some(vpn),
                    pt_response: optional_pt_response,
                    tlb_response: optional_tlb_response,
                }
            },
        };

        let (pt_response, tlb_response) = (translation_response.pt_response, translation_response.tlb_response);

        // create the physical addr from the ppn and page offset
        let physical_addr = bits::join_at(translation_response.ppn, translation_response.page_offset, self.config.pt.offset_size);
        let access_event = AccessEvent::from_raw(raw_access_type, physical_addr)?;

        /* Step 2: Try to access data in caches in the order of DC -> L2 -> Memory */

        let dc_response = match access_event {
            AccessEvent::Read(addr) => self.dc.read(addr),
            AccessEvent::Write(addr) => self.dc.write(addr),
        };
        if let Some(writeback_addr) = dc_response.writeback {
            self.l2.write_force(writeback_addr);
        }

        let l2_response: Option<cache::CacheResponse> = if self.config.l2.enabled {
            match dc_response.result {
                // If DC has a write through policy, then we write through to L2
                QueryResult::Hit if access_event.is_write() && self.config.dc.write_policy == WriteThrough => {
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

        if let Some(l2) = &l2_response {
            // Write back to main memory
            if let Some(writeback) = l2.writeback {
                self.pt.touch(writeback)
            }
            // Invalidate entries in L1 if there was an eviction from L2
            if let Some(evicted_addr) = l2.eviction {
                self.dc.clean_addr(evicted_addr);
            }
        }

        let mem_response = MemoryResponse {
            addr: raw_addr,
            page_offset: translation_response.page_offset,
            vpn: translation_response.vpn,
            ppn: translation_response.ppn,
            page_table_res: pt_response.as_ref().map(|r| r.res),
            tlb_tag: tlb_response.as_ref().map(|r| r.tag),
            tlb_idx: tlb_response.as_ref().map(|r| r.idx),
            tlb_res: tlb_response.as_ref().map(|r| r.result),
            dc_tag: dc_response.tag,
            dc_idx: dc_response.idx,
            dc_res: Some(dc_response.result),
            l2_tag: l2_response.as_ref().map(|r| r.tag),
            l2_idx: l2_response.as_ref().map(|r| r.idx),
            l2_res: l2_response.as_ref().map(|r| r.result),
            ..Default::default()
        };

        Ok(mem_response)
    }
}

/// Details the interior behavior of a simulated access to the memory system.
#[derive(Default)]
pub struct MemoryResponse {
    addr: u32,
    vpn: Option<u32>,
    ppn: u32,
    page_offset: u32,
    tlb_tag: Option<u32>,
    tlb_idx: Option<u32>,
    tlb_res: Option<QueryResult>,
    page_table_res: Option<QueryResult>,

    dc_tag: u32,
    dc_idx: u32,
    dc_res: Option<QueryResult>,
    l2_tag: Option<u32>,
    l2_idx: Option<u32>,
    l2_res: Option<QueryResult>,
}

impl std::fmt::Display for MemoryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, 
            //addr  pg # pgoff tbtg tbix tlbr ptrs phypg dctag dcidx dcrs l2tg l2ix l2rs
            "{:08x} {:6} {:4x} {:6} {:3} {:4} {:4} {:4x} {:6x} {:3x} {:4} {:6} {:3} {:4}",

            self.addr,
            self.vpn.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.page_offset,
            self.tlb_tag.map_or("".to_string(), |n| format!("{:6x}", n)),
            self.tlb_idx.map_or("".to_string(), |n| format!("{:3x}", n)),
            self.tlb_res.as_ref().map_or("", |q| q.as_str()),
            self.page_table_res.as_ref().map_or("", |q| q.as_str()),
            self.ppn,
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