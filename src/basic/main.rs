use wasm_bindgen::prelude::*;

use crate::basic::localtrust::{ canonicalize_local_trust };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };


pub fn run_compute() -> u64 {
    let e = 1.25e-7;
    let a = 0.5;

    let c = CSRMatrix {
        cs_matrix: CSMatrix {
            major_dim: 5,
            minor_dim: 4,
            entries: vec![
                vec![
                    Entry { index: 0, value: 100.0 },
                    Entry { index: 1, value: 200.0 },
                    Entry { index: 2, value: 300.0 }
                ],
                vec![Entry { index: 1, value: 400.0 }, Entry { index: 3, value: 500.0 }],
                vec![],
                vec![
                    Entry { index: 0, value: 600.0 },
                    Entry { index: 1, value: 700.0 },
                    Entry { index: 2, value: 800.0 },
                    Entry { index: 3, value: 900.0 }
                ],
                vec![Entry { index: 2, value: 1000.0 }]
            ],
        },
    };

    let p = Vector::new(5, vec![Entry { index: 0, value: 1.0 }]);

    let result = compute(&c, &p, a, e, None, None);

    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let result = run_compute();
        println!("{:?}", result);

        // let expected_l = 1;
        // assert_eq!(l, expected_l, "{}: l = {}", l, expected_l);
    }
}
