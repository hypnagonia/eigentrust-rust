use std::cmp::Ordering;
use std::f64;
use std::sync::{Arc, Mutex};

use super::entry::{CooEntry, Entry};
use super::matrix::CSRMatrix;
use super::util::KBNSummer;
use num_cpus;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde::Serialize;
use std::thread;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Vector {
    pub dim: usize,
    pub entries: Vec<Entry>,
}

impl Vector {
    pub fn new(dim: usize, entries: Vec<Entry>) -> Vector {
        let mut vector = Vector { dim, entries };
        vector.sort_entries_by_index();
        vector
    }

    pub fn nnz(&self) -> usize {
        self.entries.len()
    }

    pub fn assign(&mut self, other: &Vector) {
        self.dim = other.dim;
        self.entries = other.entries.clone();
    }

    pub fn clone(&self) -> Vector {
        Vector {
            dim: self.dim,
            entries: self.entries.clone(),
        }
    }

    pub fn set_dim(&mut self, dim: usize) {
        if dim < self.dim {
            self.entries.retain(|e| e.index < dim);
        }
        self.dim = dim;
    }

    pub fn sum(&self) -> f64 {
        self.entries.iter().map(|e| e.value).sum()
    }

    pub fn add_vec(&mut self, v1: &Vector, v2: &Vector) -> Result<(), String> {
        if v1.dim != v2.dim {
            return Err("Dimension mismatch".to_string());
        }

        let mut e1 = &v1.entries[..];
        let mut e2 = &v2.entries[..];
        let mut entries = Vec::with_capacity(v1.entries.len() + v2.entries.len());

        while !e1.is_empty() || !e2.is_empty() {
            let entry = match (e1.first(), e2.first()) {
                (Some(e1_first), Some(e2_first)) => match e1_first.index.cmp(&e2_first.index) {
                    Ordering::Less => {
                        let e = e1_first.clone();
                        e1 = &e1[1..];
                        e
                    }
                    Ordering::Greater => {
                        let e = e2_first.clone();
                        e2 = &e2[1..];
                        e
                    }
                    Ordering::Equal => {
                        let e = Entry {
                            index: e1_first.index,
                            value: e1_first.value + e2_first.value,
                        };
                        e1 = &e1[1..];
                        e2 = &e2[1..];
                        e
                    }
                },
                (Some(e1_first), None) => {
                    let e = e1_first.clone();
                    e1 = &e1[1..];
                    e
                }
                (None, Some(e2_first)) => {
                    let e = e2_first.clone();
                    e2 = &e2[1..];
                    e
                }
                (None, None) => {
                    break;
                }
            };

            entries.push(entry);
        }

        self.dim = v1.dim;
        self.entries = entries;
        Ok(())
    }

    pub fn sub_vec(&mut self, v1: &Vector, v2: &Vector) -> Result<(), String> {
        if v1.dim != v2.dim {
            return Err("Dimension mismatch".to_string());
        }

        let mut e1 = &v1.entries[..];
        let mut e2 = &v2.entries[..];
        let mut entries = Vec::with_capacity(v1.entries.len() + v2.entries.len());

        while !e1.is_empty() || !e2.is_empty() {
            let entry = match (e1.first(), e2.first()) {
                (Some(e1_first), Some(e2_first)) => match e1_first.index.cmp(&e2_first.index) {
                    Ordering::Less => {
                        let e = e1_first.clone();
                        e1 = &e1[1..];
                        e
                    }
                    Ordering::Greater => {
                        let e = Entry {
                            index: e2_first.index,
                            value: -e2_first.value,
                        };
                        e2 = &e2[1..];
                        e
                    }
                    Ordering::Equal => {
                        let e = Entry {
                            index: e1_first.index,
                            value: e1_first.value - e2_first.value,
                        };
                        e1 = &e1[1..];
                        e2 = &e2[1..];
                        e
                    }
                },
                (Some(e1_first), None) => {
                    let e = e1_first.clone();
                    e1 = &e1[1..];
                    e
                }
                (None, Some(e2_first)) => {
                    let e = Entry {
                        index: e2_first.index,
                        value: -e2_first.value,
                    };
                    e2 = &e2[1..];
                    e
                }
                (None, None) => {
                    break;
                }
            };

            entries.push(entry);
        }

        self.dim = v1.dim;
        self.entries = entries;
        Ok(())
    }

    pub fn scale_vec(&mut self, a: f64, v1: &Vector) {
        if a == 0.0 {
            self.dim = v1.dim;
            self.entries.clear();
            return;
        }

        if (self as *const _) != (v1 as *const _) {
            self.assign(v1);
        }

        self.scale_in_place(a);
    }

    fn scale_in_place(&mut self, a: f64) {
        if a == 1.0 {
            return;
        }

        let mut non_zero_entries = Vec::with_capacity(self.entries.len());
        for entry in &mut self.entries {
            entry.value *= a;
            if entry.value != 0.0 {
                non_zero_entries.push(entry.clone());
            }
        }

        self.entries = non_zero_entries;
    }

    pub fn norm2_bak(&self) -> f64 {
        self.entries
            .iter()
            .map(|e| e.value * e.value)
            .sum::<f64>()
            .sqrt()
    }

    pub fn norm2(&self) -> f64 {
        let mut summer = KBNSummer::new();

        for e in &self.entries {
            summer.add(e.value * e.value);
        }

        let sum =  self.entries
        .iter()
        .map(|e| e.value)
        .sum::<f64>();

        println!("norm2 entries {:?}",self.entries.len());
        println!("norm2 flat sum {:?}",sum);
        println!("norm2 sum {:?}",summer.sum());
        summer.sum().sqrt()
    }

    fn sort_entries_by_index(&mut self) {
        self.entries.sort_by_key(|e| e.index);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn mul_vec(&mut self, m: &CSRMatrix, v1: &Vector) -> Result<(), String> {
        let dim = m.cs_matrix.dim()?;
        if dim != v1.dim {
            return Err("Dimension mismatch".to_string());
        }

        let num_threads = num_cpus::get();
        let pool = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        let entries: Vec<Entry> = pool.install(|| {
            (0..dim)
                .into_par_iter()
                .filter_map(|row| {
                    let row_vector = m.row_vector(row);
                    let product = vec_dot(&row_vector, &v1);

                    if product != 0.0 {
                        Some(Entry {
                            index: row,
                            value: product,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        });

        self.dim = dim;
        self.entries = entries;

        Ok(())
    }

    // single-threaded
    #[cfg(target_arch = "wasm32")]
    pub fn mul_vec(&mut self, m: &CSRMatrix, v1: &Vector) -> Result<(), String> {
        let dim = m.cs_matrix.dim()?;
        if dim != v1.dim {
            return Err("Dimension mismatch".to_string());
        }

        let mut entries = Vec::with_capacity(dim);

        for row in 0..dim {
            let product = vec_dot(&m.row_vector(row), &v1);
            if product != 0.0 {
                entries.push(Entry {
                    index: row,
                    value: product,
                });
            }
        }

        entries.sort_by_key(|e| e.index);
        self.dim = dim;
        self.entries = entries;

        Ok(())
    }
}

pub fn vec_dot(v1: &Vector, v2: &Vector) -> f64 {
    let n2 = v2.entries.len();
    if n2 == 0 {
        return 0.0;
    }

    let mut i2 = 0;
    let mut summer = KBNSummer::new();

    for e1 in &v1.entries {
        while i2 < n2 && v2.entries[i2].index <= e1.index {
            if v2.entries[i2].index == e1.index {
                let value = e1.value * v2.entries[i2].value;
                summer.add(value);
            }
            i2 += 1;
            if i2 == n2 {
                return summer.sum();
            }
        }
    }

    summer.sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vector() {
        let v = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        assert_eq!(v.dim, 5);
        assert_eq!(v.nnz(), 2);
    }

    #[test]
    fn test_assign() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let mut v2 = Vector::new(0, vec![]);
        v2.assign(&v1);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_clone() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let v2 = v1.clone();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_set_dim() {
        let mut v = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        v.set_dim(3);
        assert_eq!(v.dim, 3);
        assert_eq!(v.nnz(), 1);
    }

    #[test]
    fn test_sum() {
        let v = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        assert_eq!(v.sum(), 3.0);
    }

    #[test]
    fn test_add_vec() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let v2 = Vector::new(
            5,
            vec![
                Entry {
                    index: 1,
                    value: 3.0,
                },
                Entry {
                    index: 3,
                    value: 4.0,
                },
            ],
        );
        let mut v3 = Vector::new(0, vec![]);
        let result = v3.add_vec(&v1, &v2);
        assert!(result.is_ok());
        let expected = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 1,
                    value: 3.0,
                },
                Entry {
                    index: 3,
                    value: 6.0,
                },
            ],
        );
        assert_eq!(v3, expected);
    }

    #[test]
    fn test_sub_vec() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let v2 = Vector::new(
            5,
            vec![
                Entry {
                    index: 1,
                    value: 3.0,
                },
                Entry {
                    index: 3,
                    value: 4.0,
                },
            ],
        );
        let mut v3 = Vector::new(0, vec![]);
        let result = v3.sub_vec(&v1, &v2);
        assert!(result.is_ok());
        let expected = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 1,
                    value: -3.0,
                },
                Entry {
                    index: 3,
                    value: -2.0,
                },
            ],
        );
        assert_eq!(v3, expected);
    }

    #[test]
    fn test_scale_vec() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let mut v2 = Vector::new(0, vec![]);
        v2.scale_vec(2.0, &v1);
        let expected = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 2.0,
                },
                Entry {
                    index: 3,
                    value: 4.0,
                },
            ],
        );
        assert_eq!(v2, expected);
    }

    #[test]
    fn test_vec_dot() {
        let v1 = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let v2 = Vector::new(
            5,
            vec![
                Entry {
                    index: 1,
                    value: 3.0,
                },
                Entry {
                    index: 3,
                    value: 4.0,
                },
            ],
        );
        let dot = vec_dot(&v1, &v2);
        assert_eq!(dot, 8.0);

        let v3 = Vector::new(
            4,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );
        let v4 = Vector::new(
            5,
            vec![
                Entry {
                    index: 1,
                    value: 3.0,
                },
                Entry {
                    index: 3,
                    value: 4.0,
                },
            ],
        );
        let dot2 = vec_dot(&v3, &v4);
        assert_eq!(dot2, 8.0);
    }

    #[test]
    fn test_norm2() {
        let v = Vector::new(
            5,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 3,
                    value: 2.0,
                },
            ],
        );

        assert!(
            (v.norm2() - (5.0_f64).sqrt()).abs() < f64::EPSILON,
            "Norm2 calculation is incorrect"
        );
    }

    #[test]
    fn test_mul_vec() {
        // Create a simple matrix
        let entries = vec![
            (0, 0, 1.0),
            (1, 2, 2.0),
            (2, 4, 3.0), //CooEntry  { row: 0, column: 0, value: 1.0 },
                         //CooEntry { row: 1, column: 2, value: 2.0 },
                         //CooEntry { row: 2, column: 4, value: 3.0 },
        ];

        let matrix = CSRMatrix::new(3, 3, entries);

        let v1 = Vector::new(
            3,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 2,
                    value: 2.0,
                },
                Entry {
                    index: 4,
                    value: 3.0,
                },
            ],
        );

        let mut v2 = Vector::new(0, vec![]); // Start with an empty vector

        let result = v2.mul_vec(&matrix, &v1);

        assert!(result.is_ok());

        let expected = Vector::new(
            3,
            vec![
                Entry {
                    index: 0,
                    value: 1.0,
                },
                Entry {
                    index: 1,
                    value: 4.0,
                },
                Entry {
                    index: 2,
                    value: 9.0,
                },
            ],
        );

        assert_eq!(v2, expected);
    }
}
