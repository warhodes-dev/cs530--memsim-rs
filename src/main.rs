/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */
pub mod config;
pub mod trace;
pub mod utils;

use crate::{
    config::Config,
    /*
    trace::{
        TraceReader,
        TraceRef,
    }
    */
};

fn main() {
    let config_path = "./trace.config";
    let config = match Config::from_file(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config: {e}");
            return;
        }
    };
    println!("{}", config);

    /*
    let trace_path = "../memhier/long_trace.dat";
//  let trace_reader = match TraceReader::from_stdin() {
    let trace_reader = match TraceReader::from_file(trace_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error reading trace from file: {e}");
//          eprintln!("Error reading trace from stdin: {e}");
            return;
        }
    };


    println!("TRACE SUCCESSFULLY LOADED:");
    for trace_event in trace_reader {
        match trace_event {
            TraceRef::Read(e) => println!("rd: {:#08x}", e),
            TraceRef::Write(e) => println!("wr: {:#08x}", e),
        }
    }
    */
}
