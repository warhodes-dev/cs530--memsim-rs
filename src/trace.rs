use std::fs::File;
use std::error::Error;
use std::io::{self, prelude::*, BufReader};

#[derive(Debug)]
pub enum TraceRef {
    Read(u32),
    Write(u32),
}

pub struct TraceReader;

impl TraceReader {
    pub fn from_stdin(stdin_lock: io::StdinLock) -> Result<impl Iterator<Item = TraceRef> + '_, Box<dyn Error>> {
        let lines = stdin_lock.lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        let trace_refs = lines.filter_map(|line| -> Option<TraceRef> {
            let idx = line.find(':').map(|i| i+1).unwrap();
            let (access_type_str, access_addr_str) = line.split_at(idx);

            let access_type = access_type_str.chars().next().unwrap();
            let access_addr = u32::from_str_radix(access_addr_str, 16).unwrap();

            match access_type {
                'R' | 'r' => Some(TraceRef::Read(access_addr)),
                'W' | 'w' => Some(TraceRef::Write(access_addr)),
                _ => None,
            }
        });

        Ok(trace_refs)
    }

    pub fn from_file(path: &str) -> Result<impl Iterator<Item = TraceRef>, Box<dyn Error>> {
        let file = File::open(path)?;

        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));

        let trace_refs = lines.filter_map(|line| -> Option<TraceRef> {
            let idx = line.find(':').map(|i| i+1).unwrap();
            let (access_type_str, access_addr_str) = line.split_at(idx);

            let access_type = access_type_str.chars().next().unwrap();
            let access_addr = u32::from_str_radix(access_addr_str, 16).unwrap();

            match access_type {
                'R' | 'r' => Some(TraceRef::Read(access_addr)),
                'W' | 'w' => Some(TraceRef::Write(access_addr)),
                _ => None,
            }
        });

        Ok(trace_refs)
    }

}
