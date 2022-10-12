/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */
use memsim_rs::config::Config;
use memsim_rs::memory::Memory;
use std::io::BufRead;
use std::env;

const TABLE_HEADER: &str =
     /*Type*/"Virt.  Page TLB    TLB TLB  PT   Phys        DC  DC          L2  L2  \n\
     Address  Page # Off  Tag    Ind Res. Res. Pg # DC Tag Ind Res. L2 Tag Ind Res.\n\
     -------- ------ ---- ------ --- ---- ---- ---- ------ --- ---- ------ --- ----";

/// Read the trace file in from stdin. Produces an iterator of tuples of `char` and `u32`,
/// which can be thought of as ('r' | 'w', addr) 
pub fn trace_from_stdin(
    stdin_lock: std::io::StdinLock
) -> Result<impl Iterator<Item = (char, u32)> + '_, Box<dyn std::error::Error>> {
    let lines = stdin_lock.lines()
        .filter_map(|line| line.ok());

    let trace_refs = lines.filter_map(|line| -> Option<(char, u32)> {
        let (access_type_str, access_addr_str) = line.split_at(2);

        let access_type = access_type_str.chars().next().ok_or("bad trace char");
        let access_addr = u32::from_str_radix(access_addr_str, 16);

        if access_type.is_ok() && access_addr.is_ok() {
            Some((access_type.unwrap(), access_addr.unwrap()))
        } else {
            None
        }
    });

    Ok(trace_refs)
}

fn main() {
    let config_path = match env::var("MEMSIM_CONFIG") {
        Ok(cfg) => cfg,
        Err(_) => "./trace.config".to_string(),
    };

    let config = match Config::from_file(&config_path) {
        Ok(c) => {
            println!("{}", c);
            c
        },
        Err(e) => {
            eprintln!("Error reading config: {e}");
            return;
        }
    };

    let addr_type = config.address_type;

    let mut mem = Memory::new(config);

    // Oddity: Returning 'static handles was not added until rustc 
    // version 1.61.0. For now, we must instantiate the stdin lock 
    // in main before passing it into `trace_from_stdin(...)` (since 
    // `trace_from_stdin(...)` returns a iterator over the stdin buf).
    let stdin = std::io::stdin();
    let stdin_lock = stdin.lock();
    let trace_reader = trace_from_stdin(stdin_lock)
        .expect("Error reading from stdin");

    println!("{} {}", addr_type.as_str(), TABLE_HEADER);
    for (trace_char, trace_addr) in trace_reader {
        let access_result = mem.access(trace_char, trace_addr);
        match access_result {
            Ok(access) => {
                print!("{}", access);
                println!()
            }
            Err(e) => {
                eprintln!("Invalid access: {}", e);
                return;
            }
        }
    }
}
