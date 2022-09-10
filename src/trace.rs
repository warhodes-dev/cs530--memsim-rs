use std::fs::File;
use std::error::Error;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
pub enum TraceRef {
    Read(u32),
    Write(u32),
}

pub struct TraceReader {
    refs: Vec<TraceRef>,
}

impl TraceReader {
    pub fn from_file(path: &str) -> Result<TraceReader, Box<dyn Error>> {
        let badline_err = |line: &str| {
            Err( format!("Malformed access in trace file {}", line).into() )
        };

        let file = File::open(path)?;

        let lines = BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.is_empty() && line.contains(':'));
                
        
        let mut refs = Vec::<TraceRef>::new();
        for line in lines {
            if let Some(i) = line.find(':').map(|i| i+1) {
                let (access_type_str, access_addr_str) = line.split_at(i);

                let access_type = access_type_str.chars().next().unwrap();
                let access_addr = u32::from_str_radix(access_addr_str, 16)?;

                let access = match access_type {
                    'R' | 'r' => TraceRef::Read(access_addr),
                    'W' | 'w' => TraceRef::Write(access_addr),
                    _ => { return badline_err(&line); }
                };

                refs.push(access);
            }
        }

        Ok( TraceReader { refs })
    }
}

impl core::ops::Deref for TraceReader {
    type Target = Vec<TraceRef>;

    fn deref(&self) -> &Self::Target {
        &self.refs
    }
}
