/* Austin Rhodes
 * PA 1: Memory Hierarchy Simulation
 * COSC 530 -- Fall 2022 */

use std::error::Error;
use memsim_rs::config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = "../trace.config";
    let config = Config::from_file(config_path)?;
    println!("CONFIG SUCCESSFULLY CREATED:\n{:#?}", config);
    Ok(())
}
