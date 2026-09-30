//! cargo run --example basic

use eigentrust::{eigentrust, PreTrust, TrustEdge};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let names = ["alice", "bob", "carol", "mallory"];

    // who trusts whom, and how much (weights are relative per truster)
    let local_trust = [
        TrustEdge::new(0, 1, 3.0), // alice -> bob
        TrustEdge::new(0, 2, 1.0), // alice -> carol
        TrustEdge::new(1, 2, 2.0), // bob -> carol
        TrustEdge::new(2, 0, 1.0), // carol -> alice
        TrustEdge::new(3, 3, 9.0), // mallory only vouches for herself
    ];
    // alice is trusted from the start
    let pre_trust = [PreTrust::new(0, 1.0)];

    let result = eigentrust(local_trust, pre_trust)?;

    for (peer, score) in result.ranking() {
        println!("{:<8} {:.4}", names[peer], score);
    }
    println!("converged after {} iterations", result.iterations());
    Ok(())
}
