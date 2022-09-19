use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::error::Error;

use crate::utils::{error, bit_ops};

const MAX_TLB_SETS: usize = 256;
const MAX_TLB_ASSOC: usize = 8;
const MAX_VIRT_PAGES: usize = 8192;
const MAX_PHYS_PAGES: usize = 1024;
const MAX_DC_SETS: usize = 8192;
const MAX_DC_ASSOC: usize = 8;
const MIN_DC_LINE_SIZE: usize = 8;
const MAX_L2_ASSOC: usize = 8;
const MIN_L2_LINE_SIZE: usize = MIN_DC_LINE_SIZE;
#[allow(dead_code)]
const MAX_REF_ADDR_LEN: u32 = 32;


#[derive(Copy, Clone, Debug)]
pub struct TLBConfig {
    pub sets: usize,
    pub set_entries: usize,
    pub idx_size: usize,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableConfig {
    pub virtual_pages: usize,
    pub physical_pages: usize,
    pub page_size: usize,
    pub idx_size: usize,
    pub offset_size: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct CacheConfig {
    pub sets: usize,
    pub set_entries: usize,
    pub line_size: usize,
    pub idx_size: usize,
    pub offset_size: usize,
    pub walloc_enabled: bool,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum AddressType {
    Physical,
    Virtual,
}

impl AddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Physical => "Physical",
            Self::Virtual => "Virtual",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub tlb: TLBConfig,
    pub pt: PageTableConfig,
    pub dc: CacheConfig,
    pub l2: CacheConfig,
    pub address_type: AddressType,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path)?;

        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        // Assume config file is always in correct order
        let mut opts = Vec::<String>::new();
        for line in lines {
            if let Some(i) = line.find(':').map(|i| i+1)  {
                let (_, opt) = line.split_at(i + 1);
                opts.push(opt.trim().to_owned());
            }
        }

        if opts.len() != 16 {
            error!("Expected 16 configuration parameters, found {}.", opts.len());
        }

        let tlb_config = {
            let sets = opts[0].parse::<usize>()?;
            let set_entries = opts[1].parse::<usize>()?;
            let idx_size   = bit_ops::min_bits(sets as u32) as usize;
		    let enabled = opts[14] == "y";
            
            if sets > MAX_TLB_SETS {
                error!("{} TLB sets specified but max is {}.", sets, MAX_TLB_SETS);
            }
            if set_entries > MAX_TLB_ASSOC {
                error!("TLB has associativity of {} but max is {}.", set_entries, MAX_TLB_ASSOC);
            }
            if set_entries.count_ones() != 1 {
                error!("TLB associativity is {} but must be a power of 2", set_entries);
            }

            TLBConfig { sets, set_entries, idx_size, enabled }
        };


        let pt_config = {
            let virtual_pages = opts[2].parse::<usize>()?;
            let physical_pages = opts[3].parse::<usize>()?;
            let page_size = opts[4].parse::<usize>()?;
            let idx_size = bit_ops::min_bits(virtual_pages as u32) as usize;
            let offset_size = bit_ops::min_bits(page_size as u32) as usize;
		    //let enabled = opts[13] == "y";

            if virtual_pages > MAX_VIRT_PAGES {
                error!("The number of virtual pages is {} but max is {}.", virtual_pages, MAX_VIRT_PAGES);
            }
            if physical_pages > MAX_PHYS_PAGES {
                error!("The number of physical pages is {} but max is {}.", physical_pages, MAX_VIRT_PAGES);
            }
            if !bit_ops::is_pow2(virtual_pages as u32) {
                error!("# of virtual pages is {} but must be a power of 2", virtual_pages);
            }
            if !bit_ops::is_pow2(virtual_pages as u32) {
                error!("Page size is {} but must be a power of 2", page_size);
            }

            PageTableConfig {
                virtual_pages,
                physical_pages,
                page_size,
                idx_size,
                offset_size,
            }
        };

        let dc_config = {
            let sets = opts[5].parse::<usize>()?;
            let set_entries = opts[6].parse::<usize>()?;
            let line_size = opts[7].parse::<usize>()?;
            let idx_size = bit_ops::min_bits(sets as u32) as usize;
            let offset_size = bit_ops::min_bits(line_size as u32) as usize;
		    let walloc_enabled = opts[8] != "y";

            if sets > MAX_DC_SETS {
                error!("{} DC sets specified but max is {}", sets, MAX_DC_SETS);
            }
            if set_entries > MAX_DC_ASSOC {
                error!("DC has associativity of {} but max is {}", set_entries, MAX_DC_ASSOC);
            }
            if line_size < 8 {
                error!("DC line size is {} but minimum is {}", line_size, MIN_DC_LINE_SIZE)
            }
            if set_entries.count_ones() != 1 {
                error!("DC associativity is {} but must be a power of 2", set_entries);
            }
            if line_size.count_ones() != 1 {
                error!("DC line size is {} but must be a power of 2", line_size);
            }

            CacheConfig {
                sets,
                set_entries,
                line_size,
                idx_size,
                offset_size,
                walloc_enabled,
                enabled: true,
            }
        };

        let l2_config = {
            let sets = opts[9].parse::<usize>()?;
            let set_entries = opts[10].parse::<usize>()?;
            let line_size = opts[11].parse::<usize>()?;
            let idx_size = bit_ops::min_bits(sets as u32) as usize;
            let offset_size = bit_ops::min_bits(line_size as u32) as usize;
		    let walloc_enabled = opts[12] != "y";
		    let enabled = opts[15] == "y";

            if set_entries > MAX_L2_ASSOC {
                error!("L2 cache has associativity of {} but max is {}", set_entries, MAX_L2_ASSOC);
            }
            if line_size < 8 {
                error!("L2 line size is {} but minimum is {}", line_size, MIN_L2_LINE_SIZE)
            }
            if set_entries.count_ones() != 1 {
                error!("DC associativity is {} but must be a power of 2", set_entries);
            }
            if line_size.count_ones() != 1 {
                error!("DC line size is {} but must be a power of 2", line_size);
            }

            CacheConfig {
                sets,
                set_entries,
                line_size,
                idx_size,
                offset_size,
                walloc_enabled,
                enabled,
            }
        };

		let address_type = match opts[13].as_str() {
            "y" => AddressType::Virtual,
            "n" => AddressType::Physical,
            s => error!("Field 13 (virutal addresses enabled) must be 'y' or 'n' but was {}", s),
        };

        Ok(Config{
            tlb: tlb_config, 
            pt: pt_config, 
            dc: dc_config, 
            l2: l2_config,
            address_type,
        })
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        writeln!(f, "Data TLB contains {} sets.",    self.tlb.sets)?;
        writeln!(f, "Each set contains {} entries.", self.tlb.set_entries)?;
        writeln!(f, "Number of bits used for the index is {}.", self.tlb.idx_size)?;
        writeln!(f)?;

        writeln!(f, "Number of virtual pages is {}.", self.pt.virtual_pages)?;
        writeln!(f, "Number of physical pages is {}.", self.pt.physical_pages)?;
        writeln!(f, "Each page contains {} bytes.", self.pt.page_size)?;
        writeln!(f, "Number of bits used for the page table index is {}.", self.pt.idx_size)?;
        writeln!(f, "Number of bits used for the page offset is {}.", self.pt.offset_size)?;
        writeln!(f)?;

        writeln!(f, "D-cache contains {} sets.", self.dc.sets)?;
        writeln!(f, "Each set contains {} entries.", self.dc.set_entries)?;
        writeln!(f, "Each line is {} bytes.", self.dc.line_size)?;
        writeln!(f, "The cache {} a write-allocate and write-back policy.", 
                if self.dc.walloc_enabled { "uses" } else { "does not use" })?;
        writeln!(f, "Number of bits used for the index is {}.", self.dc.idx_size)?;
        writeln!(f, "Number of bits used for the offset is {}.", self.dc.offset_size)?;
        writeln!(f)?;

        writeln!(f, "L2-cache contains {} sets.", self.l2.sets)?;
        writeln!(f, "Each set contains {} entries.", self.l2.set_entries)?;
        writeln!(f, "Each line is {} bytes.", self.l2.line_size)?;
        writeln!(f, "The cache {} a write-allocate and write-back policy.", 
                if self.l2.walloc_enabled { "uses" } else { "does not use" })?;
        writeln!(f, "Number of bits used for the index is {}.", self.l2.idx_size)?;
        writeln!(f, "Number of bits used for the offset is {}.", self.l2.offset_size)?;
        writeln!(f)?;

        writeln!(f, "The addresses read in are {} addresses.", self.address_type.as_str().to_lowercase())?;

        if !self.tlb.enabled {
            writeln!(f, "TLB is disabled in this configuration.")?;
        }

        if !self.l2.enabled {
            writeln!(f, "L2 cache is disabled in this configuration")?;
        }
        Ok(())
    }
}












//
