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
