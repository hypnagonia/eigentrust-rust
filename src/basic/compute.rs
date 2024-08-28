use wasm_bindgen::prelude::*;

use crate::basic::localtrust::{ canonicalize_local_trust };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };


pub fn run_compute() -> Result<Vector, String> {
    let e = 1.25e-7;
    let a = 0.5;

    let c = CSRMatrix {
        cs_matrix: CSMatrix {
            major_dim: 4,
            minor_dim: 4,
            entries: vec![
                vec![
                    Entry { index: 1, value: 1.0 },
                ],
                vec![
                    Entry { index: 2, value: 1.0 },
                ],
                vec![
                    Entry { index: 0, value: 0.3333333333333333 },
                    Entry { index: 1, value: 0.3333333333333333 },
                    Entry { index: 2, value: 0.3333333333333333 },

                ],
                vec![
                    Entry { index: 0, value: 0.3333333333333333 },
                    Entry { index: 1, value: 0.3333333333333333 },
                    Entry { index: 2, value: 0.3333333333333333 },

                ],
            ],
        },
    };

    let p = Vector::new(4, vec![
        Entry { index: 0, value: 0.3333333333333333 },
        Entry { index: 1, value: 0.3333333333333333 },
        Entry { index: 2, value: 0.3333333333333333 },
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

        let l = 2;
        let expected_l = 1;
        assert_eq!(l, expected_l, "{}: l = {}", l, expected_l);
    }
}
