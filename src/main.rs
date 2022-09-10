/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */

use std::error::Error;
use memsim_rs::{
    config::Config,
    trace::{
        TraceReader,
        TraceRef,
    }
};


fn main() -> Result<(), Box<dyn Error>> {
    let config_path = "../memhier/trace.config";
    let config = Config::from_file(config_path)?;
    println!("CONFIG SUCCESSFULLY CREATED:\n{:#?}", config);

    let trace_path = "../memhier/long_trace.dat";
    let trace_reader = TraceReader::from_file(trace_path)?;
    println!("TRACE SUCCESSFULLY LOADED:");
    for item in trace_reader.iter() {
        match item {
            TraceRef::Read(a) => println!("rd: {:#08x}", a),
            TraceRef::Write(a) => println!("wr: {:#08x}", a),
        }
    }

    Ok(())
}
