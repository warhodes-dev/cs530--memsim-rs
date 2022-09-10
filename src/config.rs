use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::error::Error;

#[derive(Default, Debug)]
pub struct Config {
    /* TLB configuration */
    pub tlb_sets: u32,
    pub tlb_set_size: u32,

    /* Page Table configuration */
    pub virtual_pages: u32,
    pub physical_pages: u32,
    pub page_size: u32,

    /* Data (L1) Cache configuration */
    pub dc_sets: u32,
    pub dc_set_size: u32,
    pub dc_line_size: u32,

    /* L2 Cache configuration */
    pub l2_sets: u32,
    pub l2_set_size: u32,
    pub l2_line_size: u32,

    /* Boolean parameters */
    pub dc_write_alloc_enabled: bool,
    pub l2_write_alloc_enabled: bool,
    pub virtual_addrs_enabled: bool,
    pub tlb_enabled: bool,
    pub l2_cache_enabled: bool,
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
            return Err( format!("Expected 16 configuration parameters, \
                                Found {}", opts.len()).into())
        }

        // Assume config file is always in the same order
        Ok( Config {
            tlb_sets:       opts[0].parse::<u32>()?,
            tlb_set_size:   opts[1].parse::<u32>()?,
            virtual_pages:  opts[2].parse::<u32>()?,
            physical_pages: opts[3].parse::<u32>()?,
            page_size:      opts[4].parse::<u32>()?,
            dc_sets:        opts[5].parse::<u32>()?,
            dc_set_size:    opts[6].parse::<u32>()?,
            dc_line_size:   opts[7].parse::<u32>()?,
            l2_sets:        opts[9].parse::<u32>()?,
            l2_set_size:    opts[10].parse::<u32>()?,
            l2_line_size:   opts[11].parse::<u32>()?,

            dc_write_alloc_enabled: opts[8] == "y",
            l2_write_alloc_enabled: opts[12] == "y",
            virtual_addrs_enabled:  opts[13] == "y",
            tlb_enabled:            opts[14] == "y",
            l2_cache_enabled:       opts[15] == "y",
        })

    }
}
