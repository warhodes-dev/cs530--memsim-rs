use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::error::Error;

use crate::utils::error;

const MAX_TLB_SETS: u32 = 256;
const MAX_DC_SETS: u32 = 8192;
const MAX_ASSOC: u32 = 8;
const MAX_VIRT_PAGES: u32 = 8192;
const MAX_PHYS_PAGES: u32 = 1024;
const MAX_REF_ADDR_LEN: u32 = 32;
const MIN_DC_LINE_SZ: u32 = 8;
const MIN_L2_LINE_SIZE: u32 = MIN_DC_LINE_SZ;

#[derive(Debug)]
pub struct TLBConfig {
    pub sets: u32,
    pub set_size: u32,
    pub idx_size: u32,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct PageTableConfig {
    pub virtual_pages: u32,
    pub physical_pages: u32,
    pub page_size: u32,
    pub idx_size: u32,
    pub offset_size: u32,
    pub virtual_addrs_enabled: bool,
}

#[derive(Debug)]
pub struct CacheConfig {
    pub sets: u32,
    pub set_size: u32,
    pub line_size: u32,
    pub idx_size: u32,
    pub offset_size: u32,
    pub assoc: u32,
    pub walloc_enabled: bool,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct Config {
    tlb: TLBConfig,
    pt: PageTableConfig,
    dc: CacheConfig,
    l2: CacheConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path)?;

        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        let mut opts = Vec::<String>::new();
        for line in lines {
            if let Some(i) = line.find(':').map(|i| i+1)  {
                let (_, opt) = line.split_at(i + 1);
                opts.push(opt.trim().to_owned());
            }
        }

        if opts.len() != 16 {
            error!("Expected 16 configuration parameters, Found {}", opts.len());
        }

        // Assume config file is always in correct order
        let tlb = {
            let sets = opts[0].parse::<u32>()?;
            let set_size = opts[1].parse::<u32>()?;
            let idx_size   = 0;
		    let enabled = opts[14] == "y";
            TLBConfig { sets, set_size, idx_size, enabled }
        };

        let pt = {
            let virtual_pages = opts[2].parse::<u32>()?;
            let physical_pages = opts[3].parse::<u32>()?;
            let page_size = opts[4].parse::<u32>()?;
            let idx_size = 0;
            let offset_size = 0;
		    let virtual_addrs_enabled = opts[13] == "y";
            PageTableConfig {
                virtual_pages,
                physical_pages,
                page_size,
                idx_size,
                offset_size,
                virtual_addrs_enabled,
            }
        };

        let dc = {
            let sets = opts[5].parse::<u32>()?;
            let set_size = opts[6].parse::<u32>()?;
            let line_size = opts[7].parse::<u32>()?;
            let idx_size = 0;
            let offset_size = 0;
            let assoc = sets / set_size;
		    let walloc_enabled = opts[8] != "y";
            CacheConfig {
                sets,
                set_size,
                line_size,
                idx_size,
                offset_size,
                assoc,
                walloc_enabled,
                enabled: true,
            }
        };

        let l2 = {
            let sets = opts[9].parse::<u32>()?;
            let set_size = opts[10].parse::<u32>()?;
            let line_size = opts[11].parse::<u32>()?;
            let idx_size = 0;
            let offset_size = 0;
            let assoc = sets / set_size;
		    let walloc_enabled = opts[12] != "y";
		    let enabled = opts[15] == "y";
            CacheConfig {
                sets,
                set_size,
                line_size,
                idx_size,
                offset_size,
                assoc,
                walloc_enabled,
                enabled,
            }
        };

        let config = Config{tlb, pt, dc, l2};

        if config.tlb.sets > MAX_TLB_SETS {
            error!("{} TLB sets specified but max is {}", config.tlb.sets, MAX_TLB_SETS);
        }
        if config.dc.sets > MAX_DC_SETS {
            error!("{} DC sets specified but max is {}", config.dc.sets, MAX_DC_SETS);
        }


        Ok(config)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        writeln!(f, "Data TLB contains {} sets.",    self.tlb.sets)?;
        writeln!(f, "Each set contains {} entries.", self.tlb.set_size)?;
        //TODO: what?
        //writeln!(f, "Number of bits used for the index is {}.", self.tlb)?;
        writeln!(f)?;

        writeln!(f, "Number of virtual pages is {}.", self.pt.virtual_pages)?;
        writeln!(f, "Number of physical pages is {}.", self.pt.physical_pages)?;
        writeln!(f, "Each page contains {} bytes.", self.pt.page_size)?;
        //writeln!(f, "Number of bits used for the page table index is {}.", )?;
        //writeln!(f, "Number of bits used for the page offset is {}.", )?;
        writeln!(f)?;

        writeln!(f, "D-cache contains {} sets.", self.dc.sets)?;
        writeln!(f, "Each set contains {} entries.", self.dc.set_size)?;
        writeln!(f, "Each line is {} bytes.", self.dc.line_size)?;
        writeln!(f, "The cache {} a write-allocate and write-back policy.", 
                if self.dc.walloc_enabled { "uses" } else { "does not use" })?;
        //writeln!(f, "Number of bits used for the index is {}.", )?;
        //writeln!(f, "Number of bits used for the offset is {}.", )?;
        writeln!(f)?;

        writeln!(f, "L2-cache contains {} sets.", self.l2.sets)?;
        writeln!(f, "Each set contains {} entries.", self.l2.set_size)?;
        writeln!(f, "Each line is {} bytes.", self.l2.line_size)?;
        writeln!(f, "The cache {} a write-allocate and write-back policy.", 
                if self.l2.walloc_enabled { "uses" } else { "does not use" })?;
        //writeln!(f, "Number of bits used for the index is {}.", )?;
        //writeln!(f, "Number of bits used for the offset is {}.", )?;
        writeln!(f)?;

        writeln!(f, "The addresses read in {} virtual addresses.", 
                if self.pt.virtual_addrs_enabled { "are" } else { "are not" })?;

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
