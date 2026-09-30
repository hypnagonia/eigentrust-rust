use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;

use crate::basic::engine::calculate_from_csv;
use crate::basic::util::init_logger;
pub mod basic;
pub mod error;
pub mod sparse;

const USAGE: &str = "usage: eigentrust <localtrust.csv> <pretrust.csv> [alpha]";

fn main() -> ExitCode {
    init_logger();
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {}", message);
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (localtrust_path, pretrust_path, alpha) = match args.as_slice() {
        [lt, pt] => (lt, pt, None),
        [lt, pt, alpha] => {
            let alpha = alpha
                .parse::<f64>()
                .map_err(|_| format!("alpha {:?} is not a number\n{}", alpha, USAGE))?;
            (lt, pt, Some(alpha))
        }
        _ => return Err(USAGE.to_string()),
    };

    let read = |path: &String| {
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))
    };
    let localtrust_csv = read(localtrust_path)?;
    let pretrust_csv = read(pretrust_path)?;

    let result =
        calculate_from_csv(&localtrust_csv, &pretrust_csv, alpha).map_err(|e| e.to_string())?;

    // proper CSV so peer ids with commas or quotes round-trip
    let write = || -> csv::Result<()> {
        let mut out = csv::Writer::from_writer(io::stdout().lock());
        for (name, score) in &result {
            out.write_record([name.as_str(), &score.to_string()])?;
        }
        out.flush()?;
        Ok(())
    };
    match write() {
        // stdout closed early, e.g. `| head`
        Err(e) if matches!(e.kind(), csv::ErrorKind::Io(io) if io.kind() == io::ErrorKind::BrokenPipe) => {
            Ok(())
        }
        other => other.map_err(|e| format!("cannot write output: {}", e)),
    }
}
