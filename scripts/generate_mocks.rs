use rand::Rng;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn generate_random_weight() -> usize {
    let mut rng = rand::thread_rng();
    let random_float: f64 = rng.gen();
    (random_float * (1000 as f64 + 1.0)).floor() as usize
}

fn generate_random_int(peers_count: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=peers_count)
}

fn generate_localtrust_mock(
    localtrust_count: usize,
    peers_count: usize,
    filepath: &Path,
) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    writeln!(file, "i,j,v")?;

    for _ in 0..localtrust_count {
        let i = generate_random_int(peers_count);
        let j = generate_random_int(peers_count);
        let v = generate_random_weight();
        writeln!(file, "{},{},{}", i, j, v)?;
    }

    Ok(())
}

fn generate_pretrust_mock(
    pretrust_count: usize,
    peers_count: usize,
    filepath: &Path,
) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    writeln!(file, "i,v")?;

    for _ in 0..pretrust_count {
        let i = generate_random_int(peers_count);
        let v = generate_random_weight();
        writeln!(file, "{},{}", i, v)?;
    }

    Ok(())
}

fn main() {
    let localtrust_count: usize = env::args()
        .nth(1)
        .expect("Please provide localtrust_count")
        .parse()
        .expect("localtrust_count should be a number");
    let pretrust_count: usize = env::args()
        .nth(2)
        .expect("Please provide pretrust_count")
        .parse()
        .expect("pretrust_count should be a number");
    let peers_count: usize = env::args()
        .nth(3)
        .expect("Please provide peers_count")
        .parse()
        .expect("peers_count should be a number");

    let localtrust_filepath = Path::new("./tmp/localtrust-mock.csv");
    let pretrust_filepath = Path::new("./tmp/pretrust-mock.csv");

    if let Err(e) = generate_localtrust_mock(localtrust_count, peers_count, localtrust_filepath) {
        eprintln!("Error generating localtrust-mock.csv: {}", e);
    }

    if let Err(e) = generate_pretrust_mock(pretrust_count, peers_count, pretrust_filepath) {
        eprintln!("Error generating pretrust-mock.csv: {}", e);
    }

    println!("CSV files generated successfully.");
}
