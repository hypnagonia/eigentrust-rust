use crate::basic::engine::calculate;
pub mod basic;
pub mod sparse;

fn main() {
    let result = calculate();
    println!("{:?}", result);
}
