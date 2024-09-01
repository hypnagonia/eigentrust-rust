use super::util::current_time_millis;
use super::util::PeersMap;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{ CSMatrix, CSRMatrix };
use crate::sparse::vector::Vector;
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::f64;
use sprs::{ CsMat, CsVec, TriMat };

use super::localtrust::canonicalize_local_trust_sprs;

// Canonicalize scales sparse entries in-place so that their values sum to one.
// If entries sum to zero, Canonicalize returns an error indicating a zero-sum vector.
pub fn canonicalize(entries: &mut [Entry]) -> Result<(), String> {
    let sum: f64 = entries
        .iter()
        .map(|entry| entry.value)
        .sum();
    if sum == 0.0 {
        return Err("Zero sum vector".to_string());
    }
    for entry in entries.iter_mut() {
        entry.value /= sum;
    }
    Ok(())
}

pub struct ConvergenceChecker {
    iter: usize,
    t: Vector,
    d: f64,
    e: f64,
}

impl ConvergenceChecker {
    pub fn new(t0: &Vector, e: f64) -> ConvergenceChecker {
        ConvergenceChecker {
            iter: 0,
            t: t0.clone(),
            d: 2.0 * e, // initial sentinel
            e,
        }
    }

    pub fn update(&mut self, t: &Vector) -> Result<(), String> {
        let mut td = Vector::new(self.t.dim, vec![]);
        td.sub_vec(t, &self.t)?;

        let d = td.norm2();

        /*
        log::debug!(
            "one iteration={} log10dPace={} log10dRemaining={}",
            self.iter,
            (d / self.d).log10(),
            (d / self.e).log10()
        );
         */

        self.t.assign(t);
        self.d = d;
        self.iter += 1;
        Ok(())
    }

    pub fn converged(&self) -> bool {
        self.d <= self.e
    }

    pub fn delta(&self) -> f64 {
        self.d
    }
}

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
        entries.sort_by(|a, b| { b.value.partial_cmp(&a.value).unwrap_or(cmp::Ordering::Equal) });
        let ranking: Vec<usize> = entries
            .iter()
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

fn normalize_trimat(trimat: &mut TriMat<f64>) {
    let (rows, _cols) = trimat.shape();
    let mut row_sums = vec![0.0; rows];

    for (value, (row, _col)) in trimat.triplet_iter() {
        row_sums[row] += value;
    }

    println!("row_sums {:?}", row_sums);
    let mut normalized_trimat = TriMat::with_capacity(trimat.shape(), trimat.nnz());

    for (value, (row, col)) in trimat.triplet_iter() {
        if row_sums[row] != 0.0 {
            println!("value {} {}", value, row_sums[row]);
            normalized_trimat.add_triplet(row, col, value / row_sums[row]);
        } else {
            normalized_trimat.add_triplet(row, col, 0.0); // handle rows that sum to 0
        }
    }

    *trimat = normalized_trimat;
}

// Compute function implements the EigenTrust algorithm.
// todo Error instead of String
pub fn compute<'a>(
    mut c: &CSRMatrix,
    mut p: &Vector,
    a: f64,
    e: f64,
    max_iterations: Option<usize>,
    min_iterations: Option<usize>
) -> Result<Vector, String> {
    if a.is_nan() {
        return Err("Error: alpha cannot be NaN".to_string());
    }

    let n = c.cs_matrix.major_dim;
    if n == 0 {
        return Err("Empty local trust matrix".to_string());
    }

    if p.dim != n {
        return Err("Dimension mismatch".to_string());
    }

    log::debug!("{:?}", p.sum());

    let check_freq = 1;
    let min_iters = check_freq;

    let mut t1 = p.clone();
    let ct = c.transpose()?;

    let mut ap = p.clone();
    ap.scale_vec(a, p);

    let num_leaders = n;

    let mut conv_checker = ConvergenceChecker::new(&t1, e);

    let flat_tail = 0;
    let mut flat_tail_checker = FlatTailChecker::new(flat_tail, num_leaders);

    let mut iter = 0;
    let max_iters = max_iterations.unwrap_or(usize::MAX);
    let min_iters = min_iterations.unwrap_or(1);

    let mut pretrust_vector = CsVec::new(
        7,
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![
            0.14285714285714285,
            0.14285714285714285,
            0.14285714285714285,
            0.14285714285714285,
            0.14285714285714285,
            0.14285714285714285,
            0.14285714285714285
        ]
    );

    let mut a_pretrust = pretrust_vector.clone();
    for (_, value) in a_pretrust.iter_mut() {
        *value *= a;
    }

    let mut sl_triplet = TriMat::new((7, 7));

    sl_triplet.add_triplet(0, 1, 11.31571);
    sl_triplet.add_triplet(2, 3, 269916.08616);
    sl_triplet.add_triplet(4, 5, 3173339.366896588);
    sl_triplet.add_triplet(6, 5, 46589750.00759474);

    normalize_trimat(&mut sl_triplet);


    canonicalize_local_trust_sprs(&mut sl_triplet, Some(&pretrust_vector));

    println!("localtrust {:?}", sl_triplet);


    let localtrust_matrix: CsMat<f64> = sl_triplet.to_csr();

    let localtrust_matrix = localtrust_matrix.transpose_into();

    println!("sprx pretrust {:?}---", pretrust_vector);

    println!("sprx localtrust \n{:?}---", localtrust_matrix.to_dense());

    println!("go pretrust-----");
    for e in t1.entries.iter() {
        println!("  Index: {:?},", e);
    }

    println!("go localtrust----");
    for e in c.cs_matrix.entries.iter() {
        println!("  Index: {:?},", e);
    }

    log::info!(
        "Compute started dim={}, num_leaders={}, nnz={}, alpha={}, epsilon={}, check_freq={}",
        p.dim,
        num_leaders,
        t1.nnz(),
        a,
        e,
        check_freq
    );

    while iter < max_iters {
        log::debug!("------");

        println!("iter {:?} --------------------------", iter);
        //println!("d {:?}", conv_checker.delta());
        //println!("conv_checker.converged() {:?}", conv_checker.converged());

        if iter.saturating_sub(min_iters) % check_freq == 0 {
            if iter >= min_iters {
                conv_checker.update(&t1);

                // flat_tail_checker.update(&t1, conv_checker.delta());

                if
                    conv_checker.converged()
                    // && flat_tail_checker.reached()
                {
                    break;
                }
            }
        }

        let mut new_t1 = t1.clone(); // depr
        new_t1.mul_vec(&ct, &t1)?; // depr
        let mut t2 = new_t1.clone(); // depr
        t2.scale_vec(1.0 - a, &new_t1); // depr
        t1.add_vec(&t2, &ap)?; // depr
        
        let mut new_vector = &localtrust_matrix * &pretrust_vector;
        
       

        let mut new_vector2 = new_vector.clone();
        
        for (_, value) in new_vector2.iter_mut() {
            *value *= 1.0 - a;
        }

        let new_vector3 = new_vector2 + a_pretrust.clone();

      

        println!("new t1 {:?}", new_vector3);
        println!("go t1 {:?}", t1);


        iter += 1;
    }

    if iter >= max_iters {
        return Err("Reached maximum iterations without convergence".to_string());
    }

    log::info!(
        "finished: alpha={} dim={} nnz={} epsilon={} flatTail={} iterations={} numLeaders={}",
        a,
        n,
        ct.cs_matrix.nnz(),
        e,
        flat_tail,
        iter,
        num_leaders
    );

    Ok(t1)
}

pub fn discount_trust_vector(t: &mut Vector, discounts: &CSRMatrix) -> Result<(), String> {
    let mut i1 = 0;
    let t1 = t.clone();

    'DiscountsLoop: for (distruster, distrusts) in discounts.cs_matrix.entries.iter().enumerate() {
        'T1Loop: loop {
            if i1 >= t1.entries.len() {
                break 'DiscountsLoop;
            }
            if t1.entries[i1].index < distruster {
                i1 += 1;
                continue 'T1Loop;
            }
            if t1.entries[i1].index == distruster {
                break 'T1Loop;
            }
            if t1.entries[i1].index > distruster {
                continue 'DiscountsLoop;
            }
        }

        let scaled_distrust_vec = {
            let mut temp_vec = Vector::new(t.dim, Vec::new());
            temp_vec.scale_vec(
                t1.entries[i1].value,
                &(Vector {
                    dim: t.dim,
                    entries: distrusts.clone(),
                })
            );
            temp_vec
        };

        let t2 = t.clone();
        t.sub_vec(&t2, &scaled_distrust_vec)?;

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
                    }
                ]
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
                            }
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
                            }
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
                            }
                        ],
                        // 4 - no distrust, also zero global trust (empty)
                        vec![]
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
                    } // peer 2 & 3
                ]
            ),
        }];

        for test in test_cases {
            let mut t = test.t.clone();
            let result = discount_trust_vector(&mut t, &test.discounts);
            assert!(result.is_ok(), "{}: DiscountTrustVector failed", test.name);
            assert_eq!(t, test.expected, "{}: Vector does not match expected value", test.name);
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
                }
            ]
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
                        }
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
                        }
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
                        }
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
                        }
                    ]
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
                }
            ],
        };
        let result = compute(&c, &p, a, e, None, None).unwrap();
        assert_eq!(result, expected);
    }
}
