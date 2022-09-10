/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */
pub mod config;
pub mod trace;
pub mod utils;

use crate::{
    config::Config,
    trace::{
        TraceReader,
        TraceRef,
    }
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "../memhier/trace.config";
    let config = Config::from_file(config_path)
        .expect("Failed to open config");
    println!("{}", config);

    let trace_path = "../memhier/long_trace.dat";
    let trace_reader = TraceReader::from_file(trace_path)
    //let trace_reader = TraceReader::from_stdin();
        .expect("Failed to read from stdin");

    println!("TRACE SUCCESSFULLY LOADED:");
    for trace_event in trace_reader {
        match trace_event {
            TraceRef::Read(e) => println!("rd: {:#08x}", e),
            TraceRef::Write(e) => println!("wr: {:#08x}", e),
        }
    }

    Ok(())
}
