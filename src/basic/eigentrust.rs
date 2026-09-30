use crate::error::{Error, Result};
use crate::sparse::entry::Entry;
#[cfg(test)]
use crate::sparse::matrix::CSMatrix;
use crate::sparse::matrix::CSRMatrix;
use crate::sparse::util::KBNSummer;
use crate::sparse::vector::Vector;
use std::cmp;

// Stops the iteration only once the ranking of the top `num_leaders` peers has stayed
// the same for `length` checks. compute() uses length 0, which disables it.
pub struct FlatTailChecker {
    length: usize,
    num_leaders: usize,
    stats: FlatTailStats,
}

impl FlatTailChecker {
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

    pub fn update(&mut self, t: &Vector, d: f64) {
        let mut entries = t.entries.clone();
        entries.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(cmp::Ordering::Equal)
        });
        // only the top num_leaders positions have to stay stable
        let ranking: Vec<usize> = entries
            .iter()
            .take(self.num_leaders)
            .map(|entry| entry.index)
            .collect();

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

    pub fn reached(&self) -> bool {
        self.stats.length >= self.length
    }
}

pub struct FlatTailStats {
    pub length: usize,
    pub threshold: usize,
    pub delta_norm: f64,
    pub ranking: Vec<usize>,
}

// Upper bound on power iterations when the caller does not set one. With alpha > 0
// the error shrinks by (1 - alpha) per step; alpha = 0 on a periodic graph never converges.
pub const DEFAULT_MAX_ITERATIONS: usize = 10_000;

// Compute function implements the EigenTrust algorithm.
//
// The iteration runs on dense vectors: after a couple of iterations the trust
// vector is (almost) dense anyway, and a dense lookup turns every row dot
// product into O(nnz(row)) instead of a sparse-sparse merge walk.
pub fn compute(
    c: &CSRMatrix,
    p: &Vector,
    a: f64,
    e: f64,
    max_iterations: Option<usize>,
    min_iterations: Option<usize>,
) -> Result<Vector> {
    if !(0.0..=1.0).contains(&a) {
        return Err(Error::InvalidAlpha(a));
    }

    let n = c.cs_matrix.major_dim;
    if n == 0 {
        return Err(Error::EmptyLocalTrust);
    }

    if p.dim != n {
        return Err(Error::DimensionMismatch);
    }

    let ct = c.transpose()?;
    ct.cs_matrix.dim()?;

    let ap: Vec<f64> = p.to_dense().iter().map(|v| v * a).collect();
    let mut t1 = p.to_dense();
    let mut prev = t1.clone();
    let mut t2 = vec![0.0; n];

    let flat_tail = 0;
    let mut flat_tail_checker = FlatTailChecker::new(flat_tail, n);

    let mut iter = 0;
    let max_iters = max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
    let min_iters = min_iterations.unwrap_or(1);

    log::info!(
        "Compute started dim={}, nnz={}, alpha={}, epsilon={}",
        n,
        ct.cs_matrix.nnz(),
        a,
        e,
    );

    while iter < max_iters {
        if iter >= min_iters {
            let d = dense_delta_norm2(&t1, &prev);
            if !d.is_finite() {
                return Err(Error::NonFiniteScores);
            }
            prev.copy_from_slice(&t1);
            log::trace!("iteration={} delta={}", iter, d);

            // with flat_tail == 0 the ranking check is always satisfied,
            // skip the O(n log n) sort per iteration
            if flat_tail > 0 {
                flat_tail_checker.update(&Vector::from_dense(&t1), d);
            }

            if d <= e && flat_tail_checker.reached() {
                break;
            }
        }

        // t2 = (1 - a) * C^T * t1 + a * p
        mul_dense(&ct, &t1, &mut t2);
        for (x, ap_i) in t2.iter_mut().zip(ap.iter()) {
            *x = *x * (1.0 - a) + ap_i;
        }
        std::mem::swap(&mut t1, &mut t2);

        iter += 1;
    }

    if iter >= max_iters {
        return Err(Error::NotConverged {
            iterations: max_iters,
            alpha: a,
        });
    }

    log::info!(
        "finished: alpha={} dim={} nnz={} epsilon={} flatTail={} iterations={}",
        a,
        n,
        ct.cs_matrix.nnz(),
        e,
        flat_tail,
        iter,
    );

    Ok(Vector::from_dense(&t1))
}

// ||t - prev||2 with compensated summation, same order as the sparse version.
fn dense_delta_norm2(t: &[f64], prev: &[f64]) -> f64 {
    let mut summer = KBNSummer::new();
    for (x, y) in t.iter().zip(prev.iter()) {
        let d = x - y;
        if d != 0.0 {
            summer.add(d * d);
        }
    }
    summer.sum().sqrt()
}

// out = m * v for a square CSR matrix and a dense vector.
fn mul_dense(m: &CSRMatrix, v: &[f64], out: &mut [f64]) {
    let row_dot = |row: &[Entry]| {
        let mut summer = KBNSummer::new();
        for entry in row {
            let x = v[entry.index];
            if x != 0.0 {
                summer.add(entry.value * x);
            }
        }
        summer.sum()
    };

    #[cfg(any(not(target_arch = "wasm32"), feature = "parallel"))]
    {
        use rayon::prelude::*;
        out.par_iter_mut()
            .zip(m.cs_matrix.entries.par_iter())
            .with_min_len(1024)
            .for_each(|(o, row)| *o = row_dot(row));
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "parallel")))]
    for (o, row) in out.iter_mut().zip(m.cs_matrix.entries.iter()) {
        *o = row_dot(row);
    }
}

// Subtracts from t each peer's trust-weighted distrust row:
// t -= sum_i t[i] * discounts[i], applied in distruster order.
// Subtracts each peer's trust-weighted distrust row. Not used by calculate_from_csv, see
// extract_distrust.
pub fn discount_trust_vector(t: &mut Vector, discounts: &CSRMatrix) -> Result<()> {
    if discounts.cs_matrix.entries.iter().all(|row| row.is_empty()) {
        return Ok(());
    }

    let trust = t.to_dense();
    let mut result = trust.clone();

    for (distruster, distrusts) in discounts.cs_matrix.entries.iter().enumerate() {
        let weight = match trust.get(distruster) {
            Some(&w) if w != 0.0 => w,
            _ => continue,
        };
        for entry in distrusts {
            if entry.index >= result.len() {
                return Err(Error::DimensionMismatch);
            }
            let scaled = weight * entry.value;
            if scaled != 0.0 {
                result[entry.index] -= scaled;
            }
        }
    }

    *t = Vector::from_dense(&result);
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

        let test_cases = vec![TestCase {
            name: "test1",
            t: Vector::new(
                5,
                vec![
                    Entry {
                        index: 0,
                        value: 0.25,
                    },
                    Entry {
                        index: 2,
                        value: 0.5,
                    },
                    Entry {
                        index: 3,
                        value: 0.25,
                    },
                ],
            ),
            discounts: CSRMatrix {
                cs_matrix: CSMatrix {
                    major_dim: 5,
                    minor_dim: 5,
                    entries: vec![
                        // 0 - no distrust (empty)
                        vec![],
                        // 1 - doesn't matter because of zero trust
                        vec![
                            Entry {
                                index: 2,
                                value: 0.5,
                            },
                            Entry {
                                index: 3,
                                value: 0.5,
                            },
                        ],
                        // 2 - scaled by 0.5 and applied
                        vec![
                            Entry {
                                index: 0,
                                value: 0.25,
                            },
                            Entry {
                                index: 4,
                                value: 0.75,
                            },
                        ],
                        // 3 - scaled by 0.25 and applied
                        vec![
                            Entry {
                                index: 2,
                                value: 0.5,
                            },
                            Entry {
                                index: 4,
                                value: 0.5,
                            },
                        ],
                        // 4 - no distrust, also zero global trust (empty)
                        vec![],
                    ],
                },
            },
            expected: Vector::new(
                5,
                vec![
                    Entry {
                        index: 0,
                        value: 0.25 - 0.25 * 0.5,
                    }, // peer 2
                    Entry {
                        index: 2,
                        value: 0.5 - 0.5 * 0.25,
                    }, // peer 3
                    Entry {
                        index: 3,
                        value: 0.25,
                    },
                    Entry {
                        index: 4,
                        value: 0.0 - 0.75 * 0.5 - 0.5 * 0.25,
                    }, // peer 2 & 3
                ],
            ),
        }];

        for test in test_cases {
            let mut t = test.t.clone();
            let result = discount_trust_vector(&mut t, &test.discounts);
            assert!(result.is_ok(), "{}: DiscountTrustVector failed", test.name);
            assert_eq!(
                t, test.expected,
                "{}: Vector does not match expected value",
                test.name
            );
        }
    }

    #[test]
    fn test_run() {
        let e = 1.25e-7;
        let a = 0.5;

        let p = Vector::new(
            8,
            vec![
                Entry {
                    index: 0,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 1,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 2,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 3,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 4,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 5,
                    value: 0.14285714285714285,
                },
                Entry {
                    index: 6,
                    value: 0.14285714285714285,
                },
            ],
        );

        let c = CSRMatrix {
            cs_matrix: CSMatrix {
                major_dim: 8,
                minor_dim: 8,
                entries: vec![
                    vec![Entry {
                        index: 3,
                        value: 1.0,
                    }],
                    vec![
                        Entry {
                            index: 0,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 1,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 2,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 3,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 4,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 5,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 6,
                            value: 0.14285714285714285,
                        },
                    ],
                    vec![Entry {
                        index: 3,
                        value: 1.0,
                    }],
                    vec![
                        Entry {
                            index: 0,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 1,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 2,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 3,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 4,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 5,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 6,
                            value: 0.14285714285714285,
                        },
                    ],
                    vec![Entry {
                        index: 1,
                        value: 1.0,
                    }],
                    vec![
                        Entry {
                            index: 0,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 1,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 2,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 3,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 4,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 5,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 6,
                            value: 0.14285714285714285,
                        },
                    ],
                    vec![Entry {
                        index: 5,
                        value: 1.0,
                    }],
                    vec![
                        Entry {
                            index: 0,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 1,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 2,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 3,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 4,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 5,
                            value: 0.14285714285714285,
                        },
                        Entry {
                            index: 6,
                            value: 0.14285714285714285,
                        },
                    ],
                ],
            },
        };

        let expected = Vector {
            dim: 8,
            entries: vec![
                Entry {
                    index: 0,
                    value: 0.11111110842697292,
                },
                Entry {
                    index: 1,
                    value: 0.16666666867977029,
                },
                Entry {
                    index: 2,
                    value: 0.11111110842697292,
                },
                Entry {
                    index: 3,
                    value: 0.22222222893256766,
                },
                Entry {
                    index: 4,
                    value: 0.11111110842697292,
                },
                Entry {
                    index: 5,
                    value: 0.16666666867977029,
                },
                Entry {
                    index: 6,
                    value: 0.11111110842697292,
                },
            ],
        };
        let result = compute(&c, &p, a, e, None, None).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_alpha_zero_on_periodic_graph_stops() {
        // a <-> b with all seed trust on a oscillates forever without teleport
        let c = CSRMatrix::new(2, 2, vec![(0, 1, 1.0), (1, 0, 1.0)]);
        let p = Vector::new(2, vec![Entry::new(0, 1.0)]);
        let err = compute(&c, &p, 0.0, 1e-9, None, None).unwrap_err();
        assert!(matches!(err, Error::NotConverged { .. }), "{}", err);
        // with teleport it converges
        assert!(compute(&c, &p, 0.1, 1e-9, None, None).is_ok());
    }

    #[test]
    fn test_compute_rejects_bad_alpha() {
        let c = CSRMatrix::new(1, 1, vec![(0, 0, 1.0)]);
        let p = Vector::new(1, vec![Entry::new(0, 1.0)]);
        for a in [f64::NAN, -0.1, 1.5] {
            assert!(compute(&c, &p, a, 1e-9, None, None).is_err());
        }
    }
}
