use crate::basic::eigentrust::compute;
use crate::basic::localtrust::{canonicalize_local_trust, read_local_trust_from_csv};
use crate::basic::trustvector::canonicalize_trust_vector;
use crate::basic::trustvector::read_trust_vector_from_csv;
use crate::error::{Error, Result};
#[cfg(test)]
use std::fs;

// Ranks every peer from CSV input: `from,to[,weight]` local trust and `peer[,weight]`
// pre-trust. Returns (peer, score) sorted by score, highest first; scores sum to 1.

pub fn calculate_from_csv(
    localtrust_csv: &str,
    pretrust_csv: &str,
    alpha: Option<f64>,
) -> Result<Vec<(String, f64)>> {
    log::info!("Compute starting...");

    let a = alpha.unwrap_or(0.5);
    if !(0.0..=1.0).contains(&a) {
        return Err(Error::InvalidAlpha(a));
    }

    let (mut local_trust, peers) = read_local_trust_from_csv(localtrust_csv)?;

    let mut pre_trust = read_trust_vector_from_csv(pretrust_csv, &peers.map)?;

    let c_dim = local_trust.cs_matrix.dim()?;

    let e = 1e-6 / (c_dim as f64);

    let p_dim = pre_trust.dim;
    if c_dim < p_dim {
        local_trust.set_dim(p_dim, p_dim);
    } else {
        pre_trust.set_dim(c_dim);
    }

    canonicalize_trust_vector(&mut pre_trust);

    // negative (distrust) weights are rejected while parsing, see localtrust.rs
    canonicalize_local_trust(&mut local_trust, Some(&pre_trust))?;

    let trust_scores = compute(&local_trust, &pre_trust, a, e, None, None)?;

    let mut entries: Vec<(String, f64)> = trust_scores
        .entries
        .iter()
        .map(|e| (peers.names[e.index].clone(), e.value))
        .collect();

    entries.sort_by(|a, b| b.1.total_cmp(&a.1));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_from_csv() {
        let localtrust_csv =
            "i,j,v\nalice,bob,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474";
        let pretrust_csv =
            "i,j,v\nalice,0.14285714285714285\nbob,0.14285714285714285\n2,0.14285714285714285\n3,0.14285714285714285\n4,0.14285714285714285\n5,0.14285714285714285\n6,0.14285714285714285";
        let alpha = Some(0.5);
        let entries = calculate_from_csv(localtrust_csv, pretrust_csv, alpha).unwrap();
        assert_eq!(entries.len(), 7);
        assert!(entries[0].1 >= entries[1].1);
        assert_eq!(entries[0].0, "5");
        assert_eq!(entries[0].1, 0.22222219873601323);
        assert_eq!(entries[1].0, "bob");

        let localtrust_csv =
            "alice,bob,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474";
        let pretrust_csv = "alice,1";
        let alpha = Some(0.5);
        let entries = calculate_from_csv(localtrust_csv, pretrust_csv, alpha).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].1 >= entries[1].1);
        assert_eq!(entries[0].0, "alice");
        assert_eq!(entries[0].1, 0.6666666865348816);
    }

    #[test]
    fn test_calculate_from_csv_file() {
        let localtrust_csv = fs::read_to_string("./example/localtrust2.csv")
            .expect("Failed to read localtrust CSV file");
        let pretrust_csv = fs::read_to_string("./example/pretrust2.csv")
            .expect("Failed to read pretrust CSV file");

        let entries = calculate_from_csv(&localtrust_csv, &pretrust_csv, None).unwrap();

        assert_eq!(entries.len(), 9);
        assert!(entries[0].1 >= entries[1].1);
        assert_eq!(entries[0].0, "0x84e1056ed1b76fb03b43e924ef98833dba394b2b");
        // the file contains duplicate (i, j) records, the last one wins
        assert_eq!(entries[0].1, 0.40356129084997394);
        assert_eq!(entries[1].0, "0x9fc3b33884e1d056a8ca979833d686abd267f9f8");
    }

    fn total(entries: &[(String, f64)]) -> f64 {
        entries.iter().map(|(_, s)| s).sum()
    }

    #[test]
    fn test_scores_sum_to_one_with_duplicate_pretrust() {
        let lt = "a,b,1\nb,c,1\nc,a,1\nc,d,1";
        let pt = "a,1\nd,1\na,5\na,2";
        let entries = calculate_from_csv(lt, pt, Some(0.3)).unwrap();
        assert!((total(&entries) - 1.0).abs() < 1e-9, "{}", total(&entries));
        // last value wins: a=2, d=1 is the same as a clean 2:1 pretrust
        let clean = calculate_from_csv(lt, "a,2\nd,1", Some(0.3)).unwrap();
        assert_eq!(entries, clean);
    }

    #[test]
    fn test_rejects_bad_input() {
        for (lt, pt) in [
            ("a,b,NaN", "a"),
            ("a,b,inf", "a"),
            ("a,b,-1", "a"),
            ("a,b,1", "a,NaN"),
        ] {
            assert!(calculate_from_csv(lt, pt, None).is_err(), "{} / {}", lt, pt);
        }
    }

    #[test]
    fn test_alpha_zero_periodic_returns_error() {
        assert!(calculate_from_csv("a,b\nb,a", "a", Some(0.0)).is_err());
    }
}
