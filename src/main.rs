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

    let localtrust_csv = strip_headers(localtrust_csv);
    let pretrust_csv = strip_headers(pretrust_csv);

    let result = calculate_from_csv(&localtrust_csv, &pretrust_csv).unwrap();

    println!("{:?}", result);
}

fn strip_headers(csv_content: String) -> String {
    let mut lines = csv_content.lines();
    
    if let Some(first_line) = lines.next() {
        let fields: Vec<&str> = first_line.split(',').collect();
        if let Some(last_field) = fields.last() {
            if last_field.parse::<f64>().is_err() {
                return lines.collect::<Vec<&str>>().join("\n");
            }
        }
    }

    csv_content
}

/* 
use crate::basic::engine::calculate_from_csv;
pub mod basic;
pub mod sparse;

fn main() {
    let localtrust_csv =
        "alice,bob,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474\n";
    let pretrust_csv = "alice,0.14285714285714285\nbob,0.14285714285714285\n2,0.14285714285714285\n3,0.14285714285714285\n4,0.14285714285714285\n5,0.14285714285714285\n6,0.14285714285714285\n";
    // let pretrust_csv = "alice,0.14285714285714285\n";

    let result = calculate_from_csv(localtrust_csv, pretrust_csv);
    println!("{:?}", result);
}
*/
