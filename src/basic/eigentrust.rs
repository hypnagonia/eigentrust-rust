use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::f64;
use std::cmp;
use crate::sparse::entry::{Entry};
use crate::sparse::matrix::{CSRMatrix};
use crate::sparse::vector::{Vector, mul_vec};

// Canonicalize scales sparse entries in-place so that their values sum to one.
// If entries sum to zero, Canonicalize returns an error indicating a zero-sum vector.
pub fn canonicalize(entries: &mut [Entry]) -> Result<(), String> {
    let sum: f64 = entries.iter().map(|entry| entry.value).sum();
    if sum == 0.0 {
        return Err("Zero sum vector".to_string());
    }
    for entry in entries.iter_mut() {
        entry.value /= sum;
    }
    Ok(())
}

// ConvergenceChecker checks for convergence of trust vector series.
pub struct ConvergenceChecker {
    iter: usize,
    t: Vector,
    d: f64,
    e: f64,
}

impl ConvergenceChecker {
    // Creates a new convergence checker.
    pub fn new(t0: &Vector, e: f64) -> ConvergenceChecker {
        ConvergenceChecker {
            iter: 0,
            t: t0.clone(),
            d: 2.0 * e, // initial sentinel
            e,
        }
    }

    // Updates the checker with another iteration of the trust vector.
    pub fn update(&mut self, t: &Vector) -> Result<(), String> {
        let mut td = Vector::new(self.t.dim, vec![]);
        td.sub_vec(t, &self.t)?;
        let d = td.norm2();
        self.t.assign(t);
        self.d = d;
        self.iter += 1;
        Ok(())
    }

    // Returns true if the last updated vector has converged.
    pub fn converged(&self) -> bool {
        self.d <= self.e
    }

    // Returns the delta computed as of the last Update call.
    pub fn delta(&self) -> f64 {
        self.d
    }
}

// FlatTailChecker checks for a flat tail.
pub struct FlatTailChecker {
    length: usize,
    num_leaders: usize,
    stats: FlatTailStats,
}

impl FlatTailChecker {
    // Creates a new flat tail checker.
    pub fn new(length: usize, num_leaders: usize) -> FlatTailChecker {
        FlatTailChecker {
            length,
            num_leaders,
            stats: FlatTailStats {
                length: 0,
                threshold: 1,
                delta_norm: 1.0,
                ranking: vec![],
            },
        }
    }

    // Updates the checker with another iteration of the trust vector.
    pub fn update(&mut self, t: &Vector, d: f64) {
        let mut entries = t.entries.clone();
        entries.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(cmp::Ordering::Equal));
        let ranking: Vec<usize> = entries.iter().map(|entry| entry.index).collect();

        if ranking == self.stats.ranking {
            self.stats.length += 1;
        } else {
            if self.stats.length > 0 && self.stats.threshold <= self.stats.length {
                self.stats.threshold = self.stats.length + 1;
            }
            self.stats.length = 0;
            self.stats.delta_norm = d;
            self.stats.ranking = ranking;
        }
    }

    // Returns whether a flat tail has been reached.
    pub fn reached(&self) -> bool {
        self.stats.length >= self.length
    }
}

// FlatTailStats represents statistics about a flat tail.
pub struct FlatTailStats {
    pub length: usize,
    pub threshold: usize,
    pub delta_norm: f64,
    pub ranking: Vec<usize>,
}

// Compute function implements the EigenTrust algorithm.
pub fn compute(
    ctx: &JsValue,
    c: &CSRMatrix,
    p: &Vector,
    a: f64,
    e: f64,
    max_iterations: Option<usize>,
    min_iterations: Option<usize>
) -> Result<Vector, JsValue> {
    let n = c.cs_matrix.major_dim;
    if n == 0 {
        return Err(JsValue::from_str("Empty local trust matrix"));
    }

    if p.dim != n {
        return Err(JsValue::from_str("Dimension mismatch"));
    }

    let mut t = p.clone();
    let mut t1 = p.clone();
    let ct = c.transpose()?;
    let mut ap = p.clone();
    ap.scale_vec(a, p);

    let mut conv_checker = ConvergenceChecker::new(&t, e);
    let mut flat_tail_checker = FlatTailChecker::new(min_iterations.unwrap_or(1), n);

    let mut iter = 0;
    let max_iters = max_iterations.unwrap_or(usize::MAX);
    let min_iters = min_iterations.unwrap_or(1);

    while iter < max_iters {
        if iter >= min_iters && conv_checker.update(&t1).is_ok() && conv_checker.converged() && flat_tail_checker.reached() {
            break;
        }

        // Clone t1 to avoid mutable and immutable borrow conflicts
        let t1_clone = t1.clone();
        t1 = mul_vec(&ct, &t1_clone)?; // Perform the matrix-vector multiplication
        t1.scale_vec(1.0 - a, &t1_clone); // Use t1_clone to avoid mutable borrow conflict

        let t1_clone2 = t1.clone(); // Another clone to avoid borrow conflicts
        t1.add_vec(&t1_clone2, &ap)?;

        iter += 1;
    }

    if iter >= max_iters {
        return Err(JsValue::from_str("Reached maximum iterations without convergence"));
    }

    Ok(t1)
}

pub fn discount_trust_vector(t: &mut Vector, discounts: &CSRMatrix) -> Result<(), String> {
    let mut i1 = 0;
    let t1 = t.clone();

    for (distruster, distrusts) in discounts.cs_matrix.entries.iter().enumerate() {
        loop {
            match t1.entries.get(i1) {
                Some(entry) if entry.index < distruster => {
                    i1 += 1;
                    continue;
                }
                Some(entry) if entry.index == distruster => {
                    break;
                }
                Some(entry) if entry.index > distruster => {
                    break;
                }
                None => {
                    return Ok(());
                }
                _ => continue,
            }
        }

        let mut scaled_distrust_vec = Vector::new(t.dim, distrusts.clone());

        // To avoid borrowing conflicts, don't use `scaled_distrust_vec` as both mutable and immutable.
        let scale_value = t1.entries[i1].value;
        scaled_distrust_vec.scale_vec(scale_value, &scaled_distrust_vec.clone());

        // Similarly, avoid borrowing `t` as both mutable and immutable by using a clone of `t`.
        let t_clone = t.clone();
        t.sub_vec(&t_clone, &scaled_distrust_vec)?;

        i1 += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse::entry::Entry;
    use crate::sparse::matrix::CSRMatrix;
    use crate::sparse::vector::Vector;

    #[test]
    fn test_discount_trust_vector() {
        struct TestCase {
            name: &'static str,
            t: Vector,
            discounts: CSRMatrix,
            expected: Vector,
        }

        let test_cases = vec![
            TestCase {
                name: "test1",
                t: Vector::new(
                    5,
                    vec![
                        Entry { index: 0, value: 0.25 },
                        Entry { index: 2, value: 0.5 },
                        Entry { index: 3, value: 0.25 },
                    ],
                ),
                discounts: CSRMatrix::new(
                    5,
                    5,
                    vec![
                        // 0 - no distrust
                        // 1 - doesn't matter because of zero trust
                        (2, 1, 0.5),
                        (3, 1, 0.5),
                        // 2 - scaled by 0.5 and applied
                        (0, 2, 0.25),
                        (4, 2, 0.75),
                        // 3 - scaled by 0.25 and applied
                        (2, 3, 0.5),
                        (4, 3, 0.5),
                        // 4 - no distrust, also zero global trust
                    ],
                ),
                expected: Vector::new(
                    5,
                    vec![
                        // {index, original - distrust*gt}
                        Entry { index: 0, value: 0.25 - 0.25 * 0.5 }, // peer 2
                        Entry { index: 2, value: 0.5 - 0.5 * 0.25 },  // peer 3
                        Entry { index: 3, value: 0.25 },
                        Entry { index: 4, value: 0.0 - 0.75 * 0.5 - 0.5 * 0.25 }, // peer 2 & 3
                    ],
                ),
            },
        ];

        for test in test_cases {
            let mut t = test.t.clone();
            let result = discount_trust_vector(&mut t, &test.discounts);
            assert!(result.is_ok(), "{}: DiscountTrustVector failed", test.name);
            assert_eq!(t, test.expected, "{}: Vector does not match expected value", test.name);
        }
    }
}