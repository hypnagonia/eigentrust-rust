use crate::basic::eigentrust::compute;
use crate::basic::eigentrust::discount_trust_vector;
use crate::basic::localtrust::{
    canonicalize_local_trust, extract_distrust, read_local_trust_from_csv,
};
use crate::basic::trustvector::canonicalize_trust_vector;
use crate::basic::trustvector::read_trust_vector_from_csv;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{CSMatrix, CSRMatrix};
use crate::sparse::vector::Vector;
use std::collections::HashMap;
use std::f64::INFINITY;
use super::util::strip_headers;

pub fn calculate_from_csv(
    localtrust_csv: &str,
    pretrust_csv: &str,
    alpha: Option<f64>
) -> Result<Vec<(String, f64)>, String> {

    log::info!("Compute starting...");
    
    // let e = 1.25e-7;
    let a = alpha.unwrap_or(0.5);

    let localtrust_csv = strip_headers(localtrust_csv);
    let pretrust_csv = strip_headers(pretrust_csv);

    let (mut local_trust, peers) = read_local_trust_from_csv(&localtrust_csv).unwrap();

    let mut peer_indices = peers.map;

    let mut pre_trust = read_trust_vector_from_csv(pretrust_csv, &peer_indices).unwrap();

    let c_dim = local_trust.cs_matrix.dim().unwrap();
    let e = 1e-6 / c_dim as f64;

    let p_dim = pre_trust.dim;
    if c_dim < p_dim {
        local_trust.set_dim(p_dim, p_dim);
    } else {
        pre_trust.set_dim(c_dim);
    }

    canonicalize_trust_vector(&mut pre_trust);

    let mut discounts = extract_distrust(&mut local_trust).unwrap();

    canonicalize_local_trust(&mut local_trust, Some(pre_trust.clone())).unwrap();
    canonicalize_local_trust(&mut discounts, None).unwrap();

    let mut trust_scores = compute(&local_trust, &pre_trust, a, e, None, None).unwrap();

    // todo get rid!
    let mut trust_scores2 = trust_scores.clone();
    discount_trust_vector(&mut trust_scores2, &discounts)?;

    let mut entries = vec![];

    for e in &trust_scores.entries {
        let name_ref = peers.map_reversed.get(&e.index).unwrap();
        let name = name_ref.clone();
        entries.push((name, e.value));
    }

    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Ok(entries)
}
