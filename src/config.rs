use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::Path;
use std::error::Error;

#[derive(Default, Debug)]
pub struct Config {
    /* Data TLB configuration */
    dtlb_sets: u32,
    dtlb_set_size: u32,

    /* Page Table configuration */
    virtual_pages: u32,
    physical_pages: u32,
    page_size: u32,

    /* Data Cache configuration */
    dc_sets: u32,
    dc_set_size: u32,
    dc_line_size: u32,
    dc_write_alloc: bool,

    /* L2 Cache configuration */
    l2_sets: u32,
    l2_set_size: u32,
    l2_line_size: u32,
    l2_write_alloc: bool,

    /* Parameters */
    virt_addrs_on: bool,
    tlb_on: bool,
    l2_cache_on: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path)?;

        // Extract config parameters from file (remove whitespace etc)
        let mut opts = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'))
            .filter_map(|line| {
                line.split(':')
                    .last()
                    .map(|s| s.trim().to_owned())
            })
            .collect::<Vec<String>>();

        if opts.len() < 16 {
            return Err(format!("Expected 16 configuration parameters, \
                                Found {}", opts.len()).into())
        }

        // Assume config file is always in the same order
        Ok(Config {
            dtlb_sets:      opts[0].parse::<u32>()?,
            dtlb_set_size:  opts[1].parse::<u32>()?,
            virtual_pages:  opts[2].parse::<u32>()?,
            physical_pages: opts[3].parse::<u32>()?,
            page_size:      opts[4].parse::<u32>()?,
            dc_sets:        opts[5].parse::<u32>()?,
            dc_set_size:    opts[6].parse::<u32>()?,
            dc_line_size:   opts[7].parse::<u32>()?,
            dc_write_alloc: yn_to_bool(&opts[8]),
            l2_sets:        opts[9].parse::<u32>()?,
            l2_set_size:    opts[10].parse::<u32>()?,
            l2_line_size:   opts[11].parse::<u32>()?,
            l2_write_alloc: yn_to_bool(&opts[12]),
            virt_addrs_on:  yn_to_bool(&opts[13]),
            tlb_on:         yn_to_bool(&opts[14]),
            l2_cache_on:    yn_to_bool(&opts[15]),
        })

    }
}

fn yn_to_bool(s: &str) -> bool {
    if s == "y" { true } else { false }
}
