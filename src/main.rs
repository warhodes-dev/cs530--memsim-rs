/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */
mod config;
mod trace;
mod utils;
mod hw;

use crate::{
    config::Config,
    trace::{
        TraceReader,
        RawTrace,
    },
    hw::{
        Memory,
    }
};

#[allow(dead_code)]
const TABLE_HEADER: &str =
    "Virtual  Virt.  Page TLB    TLB TLB  PT   Phys        DC  DC          L2  L2\n\
     Address  Page # Off  Tag    Ind Res. Res. Pg # DC Tag Ind Res. L2 Tag Ind Res.\n\
     -------- ------ ---- ------ --- ---- ---- ---- ------ --- ---- ------ --- ----";

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

    // This is gross on purpose. Returning 'static handles was not
    // added until rustc version 1.61.0. For now, we must instantiate
    // the stdin lock in main before passing it into impl TraceReader
    // (since TraceReader::from__() returns a iterator over 'lines').
    let stdin = std::io::stdin();
    let stdin_lock = stdin.lock();
//  let trace_path = "../memhier/long_trace.dat";
    let trace_reader = match TraceReader::from_stdin(stdin_lock) {
//  let trace_reader = match TraceReader::from_file(trace_path) {
        Ok(t) => t,
        Err(e) => {
//          eprintln!("Error reading trace from file: {e}");
            eprintln!("Error reading trace from stdin: {e}");
            return;
        }
    };
    println!("TRACE SUCCESSFULLY LOADED:");

    let mem = Memory::new(config);

    for trace_event in trace_reader {
        println!("")
        /*
        match trace_event {
            RawTrace::Read(e) => println!("rd: {:#08x}", e),
            RawTrace::Write(e) => println!("wr: {:#08x}", e),
        }
        */
    }
}
