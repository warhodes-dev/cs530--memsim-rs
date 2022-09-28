use std::fs::File;
use std::error::Error;
use std::io::{self, prelude::*, BufReader};

#[derive(Debug)]
/// An input event to the memory simulation
/// 
/// TraceEvents are explicitly either TraceEvent::Read or TraceEvent::Write. Implicitly, 
/// the inner address is either physical or virtual depending on the program config.
pub enum TraceEvent {
    Read(u32),
    Write(u32),
}

impl TraceEvent {
    pub fn addr(&self) -> u32 {
        match self {
            TraceEvent::Read(e) => e.clone(),
            TraceEvent::Write(e) => e.clone(),
        }
    }
}

/// Reads in the trace file line by line, returning a TraceEvent for every valid line
type TraceReader = impl Iter;

impl TraceReader {
    pub fn from_stdin(stdin_lock: io::StdinLock) -> Result<impl Iterator<Item = TraceEvent> + '_, Box<dyn Error>> {
        let lines = stdin_lock.lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        let trace_refs = lines.filter_map(|line| -> Option<TraceEvent> {
            let idx = line.find(':').map(|i| i+1).unwrap();
            let (access_type_str, access_addr_str) = line.split_at(idx);

            let access_type = access_type_str.chars().next().unwrap();
            let access_addr = u32::from_str_radix(access_addr_str, 16).unwrap();

            match access_type {
                'R' | 'r' => Some(TraceEvent::Read(access_addr)),
                'W' | 'w' => Some(TraceEvent::Write(access_addr)),
                _ => None,
            }
        });

        Ok(trace_refs)
    }

    pub fn from_file(path: &str) -> Result<impl Iterator<Item = TraceEvent>, Box<dyn Error>> {
        let file = File::open(path)?;

        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        let trace_refs = lines.filter_map(|line| -> Option<TraceEvent> {
            let idx = line.find(':').map(|i| i+1).unwrap();
            let (access_type_str, access_addr_str) = line.split_at(idx);

            let access_type = access_type_str.chars().next().unwrap();
            let access_addr = u32::from_str_radix(access_addr_str, 16).unwrap();

            match access_type {
                'R' | 'r' => Some(TraceEvent::Read(access_addr)),
                'W' | 'w' => Some(TraceEvent::Write(access_addr)),
                _ => None,
            }
        });

        Ok(trace_refs)
    }

}
