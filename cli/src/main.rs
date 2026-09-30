//! eigentrust <localtrust.csv> <pretrust.csv> [alpha]
//!
//! Prints `peer,score` for every peer with a non-zero score, highest first.

use eigentrust::csv::Network;
use eigentrust::EigenTrustOptions;
use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;

const USAGE: &str = "usage: eigentrust <localtrust.csv> <pretrust.csv> [alpha]";

fn main() -> ExitCode {
    // RUST_LOG overrides, e.g. RUST_LOG=trace
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {}", message);
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (localtrust_path, pretrust_path, options) = match args.as_slice() {
        [lt, pt] => (lt, pt, EigenTrustOptions::default()),
        [lt, pt, alpha] => {
            let alpha = alpha
                .parse::<f64>()
                .map_err(|_| format!("alpha {:?} is not a number\n{}", alpha, USAGE))?;
            (lt, pt, EigenTrustOptions::default().with_alpha(alpha))
        }
        _ => return Err(USAGE.to_string()),
    };

    let read = |path: &String| {
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))
    };
    let network = Network::from_csv(&read(localtrust_path)?, &read(pretrust_path)?)
        .map_err(|e| e.to_string())?;
    let result = network.eigentrust(&options).map_err(|e| e.to_string())?;

    // proper CSV so peer names with commas or quotes round-trip
    let write = || -> csv::Result<()> {
        let mut out = csv::Writer::from_writer(io::stdout().lock());
        for (name, score) in network.ranking(&result) {
            if score > 0.0 {
                out.write_record([name, &score.to_string()])?;
            }
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
