//! cargo run --example csv --features csv

use eigentrust::csv::Network;
use eigentrust::EigenTrustOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_trust = "from,to,weight\nalice,bob,3\nalice,carol,1\nbob,carol,2\ncarol,alice,1\n";
    let pre_trust = "peer\nalice\n";

    let network = Network::from_csv(local_trust, pre_trust)?;
    let result = network.eigentrust(&EigenTrustOptions::default().with_alpha(0.2))?;

    for (name, score) in network.ranking(&result) {
        println!("{:<8} {:.4}", name, score);
    }
    Ok(())
}
