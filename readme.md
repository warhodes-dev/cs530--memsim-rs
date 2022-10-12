## CS530 Programming Assignment 1: Memory Hierarchy Simulator

Austin Rhodes
CS530 -- Fall 2022

---

# Build instructions for Shivam (or Dr Jantz):

1. Run `cargo build --release` from the root directory (the one that contains Cargo.toml)
2. The executable will be located in target/release/memsim-rs

Avoid trying `cargo run` or omitting the `--release` flag because I did not test my program under those conditions, though I think it would work fine.

My program reads the trace file from standard input, and reads the config from *your current working directory* as `./trace.config`.
(Example: `cat ./some_trace.dat | ./target/release/memsim-rs`)

Alternatively, it reads from the shell environment variable `MEMSIM_CONFIG` to set the config manually. 
(Example: `cat ./some_trace.dat | MEMSIM_CONFIG='/path/to/config' ./target/release/memsim-rs`)