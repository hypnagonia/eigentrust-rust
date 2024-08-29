use std::collections::HashMap;
use std::f64::INFINITY;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{ CSMatrix, CSRMatrix };
use crate::sparse::vector::Vector;
use crate::basic::trustvector::read_trust_vector_from_csv;
use crate::basic::localtrust::{ canonicalize_local_trust, read_local_trust_from_csv, extract_distrust };
use crate::basic::trustvector::canonicalize_trust_vector;
use crate::basic::eigentrust::compute;
use crate::basic::eigentrust::discount_trust_vector;

pub fn calculate_from_csv(localtrust_csv: &str, pretrust_csv: &str) -> Result<Vec<Entry>, String> {
    let e = 1.25e-7;
    let a = 0.5;

    let (mut local_trust, mut peer_indices) = read_local_trust_from_csv(&localtrust_csv).unwrap();  
    let mut pre_trust = read_trust_vector_from_csv(pretrust_csv, &peer_indices).unwrap();

    let c_dim = local_trust.cs_matrix.dim().unwrap();
    let p_dim = pre_trust.dim;
    if c_dim < p_dim {
        local_trust.set_dim(p_dim, p_dim);
    } else {
        pre_trust.set_dim(c_dim);
    }

    /*
    let lt_dim = local_trust.cs_matrix.entries.len();
    let mut dim = lt_dim;

    if peer_names.len() > 0 {
        let n = peer_names.len();
        if lt_dim < n {
            dim = n;
        } else if lt_dim > n {
            return Err("localTrust is larger than peerNames".into());
        }

        let pt_dim = pre_trust.entries.len();
        if pt_dim < n {
            // Resize pre_trust if needed
        } else if pt_dim > n {
            return Err("preTrust is larger than peerNames".into());
        }
    }
    */
    
    canonicalize_trust_vector(&mut pre_trust);
    
    let mut discounts = extract_distrust(&mut local_trust).unwrap();

    canonicalize_local_trust(&mut local_trust, Some(pre_trust.clone())).unwrap();
    canonicalize_local_trust(&mut discounts, None).unwrap();

    let mut trust_scores = compute(&local_trust, &pre_trust, a, e, None, None).unwrap();
    discount_trust_vector(&mut trust_scores, &discounts)?;

    /* 
    let mut entries = vec![];
    for i in 0..dim {
        let name = peer_names.as_ref().map_or_else(
            || format!("Peer {}", i),
            |names| names[i].clone(),
        );
        entries.push(Entry::new(i, name));
    }

    for e in &trust_scores {
        entries[e.index].score = e.score;
        entries[e.index].score_log = e.score.log10();
    }

    entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    */

    let mut entries = vec![];
    for e in &trust_scores.entries {
        entries.push(Entry::new(e.index, e.value));
    }

    // entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(entries) 
}
