use super::util::strip_headers;
use crate::basic::eigentrust::compute;
use crate::basic::eigentrust::discount_trust_vector;
use crate::basic::eigentrust::discount_trust_vector_sprs;
use crate::basic::localtrust::{
    canonicalize_local_trust, canonicalize_local_trust_sprs, extract_distrust,
    extract_distrust_sprs, read_local_trust_from_csv, read_local_trust_from_csv_sprs,
};
use crate::basic::trustvector::canonicalize_trust_vector;
use crate::basic::trustvector::canonicalize_trust_vector_sprs;
use crate::basic::trustvector::read_trust_vector_from_csv;
use crate::basic::trustvector::read_trust_vector_from_csv_sprs;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{CSMatrix, CSRMatrix};
use crate::sparse::vector::Vector;
use sprs::{CsMat, CsVec, TriMat};
use std::collections::HashMap;
use std::f64::INFINITY;
use std::fs;

// todo array inputs

pub fn calculate_from_csv(
    localtrust_csv: &str,
    pretrust_csv: &str,
    alpha: Option<f64>,
) -> Result<Vec<(String, f64)>, String> {
    log::info!("Compute starting...");

    let a = alpha.unwrap_or(0.5);

    let localtrust_csv = strip_headers(localtrust_csv);
    let pretrust_csv = strip_headers(pretrust_csv);

    let (mut local_trust, peers) = read_local_trust_from_csv(&localtrust_csv).unwrap();

    let (mut local_trust_s, peers) = read_local_trust_from_csv_sprs(&localtrust_csv).unwrap();

    let mut peer_indices = peers.map;

    let mut pre_trust = read_trust_vector_from_csv(pretrust_csv, &peer_indices).unwrap();

    let mut pre_trust_s = read_trust_vector_from_csv_sprs(pretrust_csv, &peer_indices).unwrap();

    let c_dim = local_trust.cs_matrix.dim().unwrap();

    let e = 1e-6 / (c_dim as f64);

    let p_dim = pre_trust.dim;
    if c_dim < p_dim {
        local_trust.set_dim(p_dim, p_dim);
    } else {
        pre_trust.set_dim(c_dim);
    }

    // todo resize sprs??
    let l_dim = local_trust_s.rows().max(local_trust_s.cols());
    let p_dim = pre_trust_s.dim();
    if l_dim < p_dim {
        let mut resized_matrix = TriMat::new((p_dim, p_dim));
        for (&value, (row, col)) in local_trust_s.triplet_iter() {
            resized_matrix.add_triplet(row, col, value);
        }
        local_trust_s = resized_matrix;
    } else if p_dim < l_dim {
        println!("\n\n\ngetting pretrust");
        let mut resized_vec = CsVec::new(
            l_dim,
            pre_trust_s.indices().to_vec(),
            pre_trust_s.data().to_vec(),
        );
        for i in 0..l_dim {
            if resized_vec.get(i).is_none() {
                resized_vec.append(i, 0.0);
            }
        }

        pre_trust_s = resized_vec.clone();
    }

    assert_eq!(
        local_trust_s.shape().0,
        pre_trust_s.dim(),
        "Dimension mismatch: TriMat has {} rows, but CsVec has {} elements.",
        local_trust_s.shape().0,
        pre_trust_s.dim()
    );

    canonicalize_trust_vector(&mut pre_trust);

    canonicalize_trust_vector_sprs(&mut pre_trust_s);

    println!("go pre_trust{:?}", pre_trust);

    println!("sprs pre_trust{:?}", pre_trust_s);

    let mut discounts = extract_distrust(&mut local_trust).unwrap();

    let mut discounts_s = extract_distrust_sprs(&mut local_trust_s).unwrap();

    canonicalize_local_trust(&mut local_trust, Some(pre_trust.clone())).unwrap();
    canonicalize_local_trust(&mut discounts, None).unwrap();

    canonicalize_local_trust_sprs(&mut local_trust_s, Some(&pre_trust_s.clone())).unwrap();
    canonicalize_local_trust_sprs(&mut discounts_s, None).unwrap();

    //println!("engine go {:?} localtrust\n", local_trust);

    //let localtrust_matrix: CsMat<f64> = local_trust_s.to_csr();
    //println!("\nengine sprs {:?} localtrust\n", localtrust_matrix.to_dense());

    let (mut trust_scores, mut global_trust_s) = compute(
        &local_trust,
        &pre_trust,
        &local_trust_s,
        &pre_trust_s,
        a,
        e,
        None,
        None,
    )
    .unwrap();

    log::debug!("trust_scores {:?}", trust_scores);
    log::debug!("global_trust_s {:?}", global_trust_s);

    let mut trust_scores2 = trust_scores.clone();
    discount_trust_vector(&mut trust_scores2, &discounts)?;
    discount_trust_vector_sprs(&mut global_trust_s, &discounts_s)?;

    let mut entries = vec![];

    /*
    for e in &trust_scores.entries {
        let name_ref = peers.map_reversed.get(&e.index).unwrap();
        let name = name_ref.clone();
        entries.push((name, e.value));
    }*/

    for (index, &value) in global_trust_s.iter() {
        if let Some(name_ref) = peers.map_reversed.get(&index) {
            let name = name_ref.clone();
            entries.push((name, value));
        }
    }

    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

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
        assert_eq!(entries[0].1, 0.4034661335389856);
        assert_eq!(entries[1].0, "0x9fc3b33884e1d056a8ca979833d686abd267f9f8");
    }
}
