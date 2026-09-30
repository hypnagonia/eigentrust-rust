use super::input::{for_each_record, parse_weight};
use crate::error::{Error, Input, RecordError, Result};
use crate::sparse::entry::Entry;
use crate::sparse::vector::Vector;
use std::collections::HashMap;

// CanonicalizeTrustVector canonicalizes the trust vector in-place,
// scaling it so that the elements sum to one,
// or making it a uniform vector that sums to one if it's a zero vector.
pub fn canonicalize_trust_vector(v: &mut Vector) {
    if !canonicalize(&mut v.entries) {
        let dim = v.dim;
        let c = 1.0 / dim as f64;
        v.entries.clear();
        for i in 0..dim {
            v.entries.push(Entry { index: i, value: c });
        }
    }
}

// Scales entries in place to sum to one. Returns false for a zero vector.
fn canonicalize(entries: &mut [Entry]) -> bool {
    let sum: f64 = entries.iter().map(|entry| entry.value).sum();
    if sum == 0.0 {
        return false;
    }
    for entry in entries.iter_mut() {
        entry.value /= sum;
    }
    true
}

// Reads `peer[,weight]` records. Weights must be finite and non-negative, default 1.
// A peer listed more than once keeps its last weight, same as local trust.
pub fn read_trust_vector_from_csv(
    input: &str,
    peer_indices: &HashMap<String, usize>,
) -> Result<Vector> {
    let mut levels: HashMap<usize, f64> = HashMap::new();
    let mut max_peer = -1;
    let mut duplicate_count = 0;

    for_each_record(input, Input::PreTrust, 1, |line, fields| {
        let record_error = |error| Error::Record {
            input: Input::PreTrust,
            line,
            error,
        };
        let peer = *peer_indices
            .get(fields[0])
            .ok_or_else(|| record_error(RecordError::UnknownPeer(fields[0].to_string())))?;
        let level = parse_weight(fields.get(1)).map_err(record_error)?;

        if levels.insert(peer, level).is_some() {
            duplicate_count += 1;
        }
        max_peer = max_peer.max(peer as isize);
        Ok(())
    })?;

    if duplicate_count > 0 {
        log::warn!(
            "Pretrust contains {} duplicate peers, the last value wins",
            duplicate_count
        );
    }

    let entries = levels
        .into_iter()
        .map(|(index, value)| Entry { index, value })
        .collect();
    Ok(Vector::new((max_peer + 1) as usize, entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peers(names: &[&str]) -> HashMap<String, usize> {
        names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.to_string(), i))
            .collect()
    }

    #[test]
    fn test_duplicate_pretrust_last_wins() {
        let v = read_trust_vector_from_csv("a,1\nb,1\na,3", &peers(&["a", "b"])).unwrap();
        assert_eq!(v.entries, vec![Entry::new(0, 3.0), Entry::new(1, 1.0)]);
    }

    #[test]
    fn test_pretrust_rejects_non_finite_and_negative() {
        // a bad first line reads as a header, so each bad line follows a valid one
        for bad in [
            "a,1\na,NaN",
            "a,1\na,inf",
            "a,1\na,-1",
            "a,1\na,x",
            "a,1\nb,1",
        ] {
            assert!(
                read_trust_vector_from_csv(bad, &peers(&["a"])).is_err(),
                "{}",
                bad
            );
        }
    }

    #[test]
    fn test_canonicalize_zero_vector_is_uniform_over_dim() {
        let mut v = Vector::new(4, vec![]);
        canonicalize_trust_vector(&mut v);
        assert_eq!(v.entries.len(), 4);
        assert!(v.entries.iter().all(|e| e.value == 0.25));
    }
}
