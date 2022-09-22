use crate::config;

pub type TlbEntry = (u32, u32);

// "index" refers to the index of the set, not set entries
#[derive(Debug)]
pub struct Tlb(
    Vec< // Sets
        Vec< // Set Entries
            Option<TlbEntry>>>); // Entry is either valid (Some) or invalid (None)

impl Tlb {
    pub fn new(cfg: &config::TLBConfig) -> Self {
        let empty_set = vec![ None ; cfg.set_entries as usize ];
        let tlb_inner = vec![ empty_set ; cfg.sets as usize ];
        Tlb(tlb_inner)
    }

    pub fn lookup(&self, tag: u32, idx: usize) -> TlbEntry {
        unimplemented!()
    }
}