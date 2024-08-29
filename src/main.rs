use std::env;
use std::fs;
use std::process;

use crate::basic::util::init_logger;
use crate::basic::engine::calculate_from_csv;
pub mod basic;
pub mod sparse;

fn main() {
    let args: Vec<String> = env::args().collect();
    init_logger();

    if args.len() < 3 {
        log::error!("Usage: {} <localtrust_csv_path> <pretrust_csv_path>", args[0]);
        process::exit(1);
    }

    let localtrust_csv_path = &args[1];
    let pretrust_csv_path = &args[2];

    let localtrust_csv = fs
        ::read_to_string(localtrust_csv_path)
        .expect("Failed to read localtrust CSV file");
    let pretrust_csv = fs
        ::read_to_string(pretrust_csv_path)
        .expect("Failed to read pretrust CSV file");

    let result = calculate_from_csv(&localtrust_csv, &pretrust_csv).unwrap();

    println!("{:?}", result);
}
