// The one EigenTrust implementation behind the public API, the CSV reader, the CLI and
// the WebAssembly bindings.

mod matrix;
mod power;

use crate::error::Input;
use crate::{EigenTrustError, EigenTrustOptions, PreTrust, TrustEdge, TrustScores};
use matrix::{CsrMatrix, Entry};

pub(crate) fn run(
    local_trust: impl IntoIterator<Item = TrustEdge>,
    pre_trust: impl IntoIterator<Item = PreTrust>,
    options: &EigenTrustOptions,
) -> Result<TrustScores, EigenTrustError> {
    options.validate()?;

    let mut peers = 0;
    let mut rows: Vec<Vec<Entry>> = Vec::new();
    for (position, edge) in local_trust.into_iter().enumerate() {
        check_weight(Input::LocalTrust, position, edge.weight)?;
        let bound = peer_bound(Input::LocalTrust, position, edge.from.max(edge.to))?;
        peers = peers.max(bound);
        if edge.from >= rows.len() {
            rows.resize_with(edge.from + 1, Vec::new);
        }
        rows[edge.from].push(Entry::new(edge.to, edge.weight));
    }

    let mut seeds: Vec<PreTrust> = Vec::new();
    for (position, seed) in pre_trust.into_iter().enumerate() {
        check_weight(Input::PreTrust, position, seed.weight)?;
        peers = peers.max(peer_bound(Input::PreTrust, position, seed.peer)?);
        seeds.push(seed);
    }

    if peers == 0 {
        return Err(EigenTrustError::EmptyNetwork);
    }
    rows.resize_with(peers, Vec::new);

    let p = normalize_pre_trust(&seeds, peers)?;
    let c = normalize_local_trust(CsrMatrix::from_rows(rows), &p)?;
    let dense_p = to_dense(&p, peers);

    let out = power::iterate(
        &c,
        &dense_p,
        options.alpha(),
        options.epsilon_for(peers),
        options.max_iterations(),
    )?;
    Ok(TrustScores::new(out.scores, out.iterations, out.residual))
}

fn check_weight(input: Input, position: usize, weight: f64) -> Result<(), EigenTrustError> {
    if !weight.is_finite() {
        return Err(EigenTrustError::NonFiniteWeight { input, position });
    }
    if weight < 0.0 {
        return Err(EigenTrustError::NegativeWeight {
            input,
            position,
            weight,
        });
    }
    Ok(())
}

// Number of peers needed to include `peer`.
fn peer_bound(input: Input, position: usize, peer: usize) -> Result<usize, EigenTrustError> {
    peer.checked_add(1)
        .ok_or(EigenTrustError::InvalidPeer { input, position })
}

// Sparse distribution over peers, sorted by peer. A repeated peer keeps its last weight;
// all-zero or empty pre-trust becomes uniform.
fn normalize_pre_trust(seeds: &[PreTrust], peers: usize) -> Result<Vec<Entry>, EigenTrustError> {
    let mut last: Vec<Option<f64>> = vec![None; peers];
    for seed in seeds {
        last[seed.peer] = Some(seed.weight);
    }
    let mut p: Vec<Entry> = last
        .into_iter()
        .enumerate()
        .filter_map(|(i, w)| w.map(|w| Entry::new(i, w)))
        .collect();

    let sum: f64 = p.iter().map(|e| e.value).sum();
    if !sum.is_finite() {
        return Err(EigenTrustError::WeightOverflow(Input::PreTrust));
    }
    if sum == 0.0 {
        let uniform = 1.0 / peers as f64;
        return Ok((0..peers).map(|i| Entry::new(i, uniform)).collect());
    }
    for e in &mut p {
        e.value /= sum;
    }
    Ok(p)
}

// Scales every row to sum to 1. A peer that trusts nobody passes its trust on according
// to pre-trust.
fn normalize_local_trust(mut c: CsrMatrix, p: &[Entry]) -> Result<CsrMatrix, EigenTrustError> {
    for row in &mut c.rows {
        let sum: f64 = row.iter().map(|e| e.value).sum();
        if !sum.is_finite() {
            return Err(EigenTrustError::WeightOverflow(Input::LocalTrust));
        }
        if sum == 0.0 {
            *row = p.to_vec();
        } else {
            for e in row.iter_mut() {
                e.value /= sum;
            }
        }
    }
    Ok(c)
}

fn to_dense(entries: &[Entry], len: usize) -> Vec<f64> {
    let mut dense = vec![0.0; len];
    for e in entries {
        dense[e.index] += e.value;
    }
    dense
}
