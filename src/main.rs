/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */
mod config;
mod trace;
mod utils;
mod memory;
use config::Config;
use trace::TraceReader;
use memory::Memory;

const TABLE_HEADER: &str =
   /*Typeof*/"Virt.  Page TLB    TLB TLB  PT   Phys        DC  DC          L2  L2  \n\
     Address  Page # Off  Tag    Ind Res. Res. Pg # DC Tag Ind Res. L2 Tag Ind Res.\n\
     -------- ------ ---- ------ --- ---- ---- ---- ------ --- ---- ------ --- ----";

fn main() {
    let config_path = "./trace.config";

    //TODO: Remove this printing last
    let config = match Config::from_file(config_path) {
        Ok(c) => {
            println!("{}", c);
            c
        },
        Err(e) => {
            eprintln!("Error reading config: {e}");
            return;
        }
    };

    let mut mem = Memory::new(config);

    // This is gross on purpose. Returning 'static handles was not
    // added until rustc version 1.61.0. For now, we must instantiate
    // the stdin lock in main before passing it into impl TraceReader
    // (since TraceReader::from__() returns a iterator over 'lines').
    let stdin = std::io::stdin();
    let stdin_lock = stdin.lock();
    let trace_reader = TraceReader::from_stdin(stdin_lock).expect("Error reading from stdin");
    println!("TRACE SUCCESSFULLY LOADED:");

    println!("{} {}", if mem.config.virtual_addresses {"Virtual "} else {"Physical"}, TABLE_HEADER);
    for trace_event in trace_reader {
        let access_result = mem.access(trace_event);
        println!("{}", access_result);
    }
}
