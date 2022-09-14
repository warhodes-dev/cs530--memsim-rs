use crate::config;

#[derive(Debug)]
struct TlbEntry {
    tag: u32,
    // TODO: change this to PhysicalAddr from page.rs
    phys_addr: u32,
}

// "index" refers to the index of the set, not set entries
#[derive(Default, Debug)]
pub struct Tlb(
    Vec< // Sets
        Vec< // Set Entries
            Option<TlbEntry>>>); // Entry is either valid (Some) or invalid (None)

impl Tlb {
    pub fn new(config: config::TLBConfig) -> Self {
        let empty_set = vec![ None ; config.set_entries ];
        let tlb = vec![

        ]
    }
}