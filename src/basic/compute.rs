use wasm_bindgen::prelude::*;
use crate::basic::localtrust::{ canonicalize_local_trust, read_local_trust_from_csv };
use crate::basic::trustvector::{ read_trust_vector_from_csv };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };
use std::collections::HashMap;
use getrandom::getrandom;

fn generate_random_f64(min: f64, max: f64) -> f64 {
    let mut buf = [0u8; 8];
    getrandom(&mut buf).expect("Failed to get random bytes");
    let rand_u64 = u64::from_le_bytes(buf);
    let scale = (rand_u64 as f64) / (u64::MAX as f64);
    min + scale * (max - min)
}

fn generate_csv_data(num_records: usize) -> (String, HashMap<String, usize>) {
    let mut csv_data = String::new();
    let mut peer_indices = HashMap::new();

    for i in 0..num_records {
        let from = (i + 1) % 1000;
        let to = (i ) % 100;
        let level = generate_random_f64(0.01, 1.0);

        csv_data.push_str(&format!("{},{},{}\n", from, to, level));

        peer_indices.entry(from.to_string()).or_insert(from);
        peer_indices.entry(to.to_string()).or_insert(to);
    }

    (csv_data, peer_indices)
}

pub fn run_compute() -> Result<Vector, String> {
    let e = 1.25e-7;
    let a = 0.5;

    // let (csv_data, peer_indices) = generate_csv_data(1000_000);

    let csv_data = "0,1,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474\n";
    let mut peer_indices = HashMap::new();

    for (index, line) in csv_data.lines().enumerate() {
        let mut fields = line.split(',');

        if let (Some(from), Some(to)) = (fields.next(), fields.next()) {
            peer_indices.entry(from.to_string()).or_insert(index);
            peer_indices.entry(to.to_string()).or_insert(index);
        }
    }
    let peer_indices2 = peer_indices.clone();

    let c2 = read_local_trust_from_csv(&csv_data, peer_indices).unwrap();
    let p2 = read_trust_vector_from_csv(&csv_data, &peer_indices2).unwrap();

    println!("{:?}\n {:?}\n", c2, p2);

    let c = CSRMatrix {
        cs_matrix: CSMatrix {
            major_dim: 8,
            minor_dim: 8,
            entries: vec![
                vec![
                    Entry { index: 3, value: 1.0 },
                ],
                vec![
                    Entry { index: 0, value: 0.14285714285714285 },
                    Entry { index: 1, value: 0.14285714285714285 },
                    Entry { index: 2, value: 0.14285714285714285 },
                    Entry { index: 3, value: 0.14285714285714285 },
                    Entry { index: 4, value: 0.14285714285714285 },
                    Entry { index: 5, value: 0.14285714285714285 },
                    Entry { index: 6, value: 0.14285714285714285 },

                ],
                vec![
                    Entry { index: 3, value: 1.0 },
                ],
                vec![
                    Entry { index: 0, value: 0.14285714285714285 },
                    Entry { index: 1, value: 0.14285714285714285 },
                    Entry { index: 2, value: 0.14285714285714285 },
                    Entry { index: 3, value: 0.14285714285714285 },
                    Entry { index: 4, value: 0.14285714285714285 },
                    Entry { index: 5, value: 0.14285714285714285 },
                    Entry { index: 6, value: 0.14285714285714285 },
                ],
                vec![
                    Entry { index: 1, value: 1.0 },
                ],
                vec![
                    Entry { index: 0, value: 0.14285714285714285 },
                    Entry { index: 1, value: 0.14285714285714285 },
                    Entry { index: 2, value: 0.14285714285714285 },
                    Entry { index: 3, value: 0.14285714285714285 },
                    Entry { index: 4, value: 0.14285714285714285 },
                    Entry { index: 5, value: 0.14285714285714285 },
                    Entry { index: 6, value: 0.14285714285714285 },
                ],
                vec![
                    Entry { index: 5, value: 1.0 },
                ],
                vec![
                    Entry { index: 0, value: 0.14285714285714285 },
                    Entry { index: 1, value: 0.14285714285714285 },
                    Entry { index: 2, value: 0.14285714285714285 },
                    Entry { index: 3, value: 0.14285714285714285 },
                    Entry { index: 4, value: 0.14285714285714285 },
                    Entry { index: 5, value: 0.14285714285714285 },
                    Entry { index: 6, value: 0.14285714285714285 },
                ],
            ],
        },
    };

    let p = Vector::new(8, vec![
        Entry { index: 0, value: 0.14285714285714285 },
        Entry { index: 1, value: 0.14285714285714285 },
        Entry { index: 2, value: 0.14285714285714285 },
        Entry { index: 3, value: 0.14285714285714285 },
        Entry { index: 4, value: 0.14285714285714285 },
        Entry { index: 5, value: 0.14285714285714285 },
        Entry { index: 6, value: 0.14285714285714285 },
    ]);

    let result = compute(&c, &p, a, e, None, None);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let result = run_compute();
        println!("{:?}", result);

        let l = 1;
        let expected_l = 2;
        assert_eq!(l, expected_l, "{}: l = {}", l, expected_l);
    }
}
