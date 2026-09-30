use eigentrust::csv::Network;
use eigentrust::EigenTrustOptions;
use serde_json::{json, Value};

/// Ranks peers from CSV bytes and returns JSON for JavaScript:
/// `{"Ok": [[peer, score], ...]}` with non-zero scores, highest first, or
/// `{"Err": "message"}`.
pub fn rank_csv_json(local_trust: &[u8], pre_trust: &[u8], alpha: f64) -> String {
    let result = match rank(local_trust, pre_trust, alpha) {
        Ok(ranking) => json!({ "Ok": ranking }),
        Err(message) => json!({ "Err": message }),
    };
    result.to_string()
}

fn rank(local_trust: &[u8], pre_trust: &[u8], alpha: f64) -> Result<Vec<Value>, String> {
    let local_trust =
        std::str::from_utf8(local_trust).map_err(|_| "local trust CSV is not valid UTF-8")?;
    let pre_trust =
        std::str::from_utf8(pre_trust).map_err(|_| "pre-trust CSV is not valid UTF-8")?;

    let network = Network::from_csv(local_trust, pre_trust).map_err(|e| e.to_string())?;
    let options = EigenTrustOptions::default().with_alpha(alpha);
    let result = network.eigentrust(&options).map_err(|e| e.to_string())?;

    Ok(network
        .ranking(&result)
        .into_iter()
        .filter(|&(_, score)| score > 0.0)
        .map(|(name, score)| json!([name, score]))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::rank_csv_json;
    use serde_json::Value;

    fn parse(json: &str) -> Value {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn ranks_and_sums_to_one() {
        let out = parse(&rank_csv_json(
            b"alice,bob,2\nbob,carol,1\n",
            b"alice\n",
            0.5,
        ));
        let ranking = out["Ok"].as_array().unwrap();
        let names: Vec<&str> = ranking.iter().map(|e| e[0].as_str().unwrap()).collect();
        assert_eq!(names, ["alice", "bob", "carol"]);
        let total: f64 = ranking.iter().map(|e| e[1].as_f64().unwrap()).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn errors_become_err() {
        for (lt, pt, alpha) in [
            (&b"a,b,NaN"[..], &b"a"[..], 0.5),
            (b"a,b,-1", b"a", 0.5),
            (b"", b"", 0.5),
            (&[0xff, 0xfe], b"a", 0.5),
            (b"a,b", b"a", f64::NAN),
            (b"a,b", b"a", 2.0),
        ] {
            let out = parse(&rank_csv_json(lt, pt, alpha));
            assert!(out["Err"].is_string(), "{}", out);
        }
    }
}
