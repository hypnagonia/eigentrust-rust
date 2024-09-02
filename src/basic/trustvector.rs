use crate::sparse::entry::Entry;
use sprs::CsVec;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

pub fn canonicalize_trust_vector_sprs(v: &mut CsVec<f64>) {
    if canonicalize_sprs(v).is_err() {
        let dim = v.dim();
        let c = 1.0 / (dim as f64);
        let mut indices = Vec::with_capacity(dim);
        let mut values = Vec::with_capacity(dim);
        for i in 0..dim {
            indices.push(i);
            values.push(c);
        }
        *v = CsVec::new(dim, indices, values);
    }
}

// Helper function to canonicalize a sparse vector in-place.
// Returns an error if the vector is a zero vector.
fn canonicalize_sprs(v: &mut CsVec<f64>) -> Result<(), &'static str> {
    let sum: f64 = v.iter().map(|(_, value)| *value).sum();

    if sum == 0.0 {
        return Err("Zero sum vector");
    }

    for (_, value) in v.iter_mut() {
        *value /= sum;
    }

    Ok(())
}

enum DuplicateHandling {
    Allow,
    Remove,
    Fail,
}

pub fn read_trust_vector_from_csv_sprs(
    input: &str,
    peer_indices: &HashMap<String, usize>,
) -> Result<CsVec<f64>, String> {
    let mut count = 0;
    let mut max_peer = 0;
    let mut indices = Vec::new();
    let mut values = Vec::new();
    let mut seen_peers = HashSet::new();
    let duplicate_handling = DuplicateHandling::Allow;
    let mut duplicate_count = 0;

    for line in input.lines() {
        count += 1;
        let fields: Vec<&str> = line.split(',').collect();

        let (peer, level) = match fields.len() {
            0 => {
                return Err(format!("Too few fields in line {}", count));
            }
            _ => {
                let peer = parse_peer_id(fields[0], peer_indices).map_err(|e| {
                    format!("Invalid peer {:?} in line {}: {}", fields[0], count, e)
                })?;
                let level = if fields.len() >= 2 {
                    parse_trust_level(fields[1]).map_err(|e| {
                        format!(
                            "Invalid trust level {:?} in line {}: {}",
                            fields[1], count, e
                        )
                    })?
                } else {
                    1.0
                };
                (peer, level)
            }
        };

        if seen_peers.contains(&peer) {
            match duplicate_handling {
                DuplicateHandling::Fail => {
                    return Err(format!("Duplicate peer {:?} in line {}", fields[0], count));
                }
                DuplicateHandling::Remove => {
                    duplicate_count += 1;
                    continue;
                }
                DuplicateHandling::Allow => {
                    duplicate_count += 1;
                }
            }
        } else {
            seen_peers.insert(peer);
        }

        if peer > max_peer {
            max_peer = peer;
        }

        indices.push(peer);
        values.push(level);
    }

    if duplicate_count > 0 {
        log::warn!("Pretrust contains {} duplicate peers", duplicate_count);
    }

    let mut combined: Vec<(usize, f64)> = indices.into_iter().zip(values.into_iter()).collect();
    combined.sort_by(|a, b| a.0.cmp(&b.0));
    let (sorted_indices, sorted_values): (Vec<usize>, Vec<f64>) = combined.into_iter().unzip();

    Ok(CsVec::new(max_peer + 1, sorted_indices, sorted_values))
}

fn parse_peer_id(peer_str: &str, peer_indices: &HashMap<String, usize>) -> Result<usize, String> {
    peer_indices
        .get(peer_str)
        .cloned()
        .ok_or_else(|| format!("Invalid peer: {}", peer_str))
}

fn parse_trust_level(level_str: &str) -> Result<f64, String> {
    level_str
        .parse::<f64>()
        .map_err(|_| format!("Invalid trust level: {}", level_str))
}
