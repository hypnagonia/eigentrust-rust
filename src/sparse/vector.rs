#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;

use super::entry::Entry;
use super::matrix::CSRMatrix;
use super::util::KBNSummer;
use crate::error::{Error, Result};

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Vector {
    pub dim: usize,
    pub entries: Vec<Entry>,
}

impl Vector {
    pub fn new(dim: usize, entries: Vec<Entry>) -> Self {
        let mut vector = Vector { dim, entries };
        vector.sort_entries_by_index();
        vector
    }

    pub fn to_dense(&self) -> Vec<f64> {
        let mut dense = vec![0.0; self.dim];
        // add, not assign: a sparse vector with repeated indices means their sum
        for e in &self.entries {
            dense[e.index] += e.value;
        }
        dense
    }

    pub fn from_dense(dense: &[f64]) -> Self {
        let entries = dense
            .iter()
            .enumerate()
            .filter(|(_, &v)| v != 0.0)
            .map(|(index, &value)| Entry { index, value })
            .collect();
        Vector {
            dim: dense.len(),
            entries,
        }
    }

    pub fn nnz(&self) -> usize {
        self.entries.len()
    }

    pub fn assign(&mut self, other: &Self) {
        *self = other.clone();
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

    pub fn add_vec(&mut self, v1: &Self, v2: &Self) -> Result<()> {
        self.binary_operation(v1, v2, |x, y| x + y)
    }

    pub fn sub_vec(&mut self, v1: &Self, v2: &Self) -> Result<()> {
        self.binary_operation(v1, v2, |x, y| x - y)
    }

    pub fn scale_vec(&mut self, a: f64, v1: &Self) -> Result<()> {
        if a.is_nan() {
            return Err(Error::InvalidAlpha(a));
        }
        if a == 0.0 {
            self.dim = v1.dim;
            self.entries.clear();
        } else {
            self.assign(v1);
            self.scale_in_place(a);
        }
        Ok(())
    }

    pub fn norm2(&self) -> f64 {
        let mut summer = KBNSummer::new();
        for e in &self.entries {
            summer.add(e.value * e.value);
        }
        summer.sum().sqrt()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn mul_vec(&mut self, m: &CSRMatrix, v1: &Self) -> Result<()> {
        let dim = m.cs_matrix.dim()?;
        if dim != v1.dim {
            return Err(Error::DimensionMismatch);
        }

        let dense = v1.to_dense();
        let entries: Vec<Entry> = (0..dim)
            .into_par_iter()
            .filter_map(|row| {
                let product = dense_dot(&m.cs_matrix.entries[row], &dense);
                if product != 0.0 {
                    Some(Entry {
                        index: row,
                        value: product,
                    })
                } else {
                    None
                }
            })
            .collect();

        self.dim = dim;
        self.entries = entries;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn mul_vec(&mut self, m: &CSRMatrix, v1: &Self) -> Result<()> {
        let dim = m.cs_matrix.dim()?;
        if dim != v1.dim {
            return Err(Error::DimensionMismatch);
        }

        let dense = v1.to_dense();
        let mut entries = Vec::with_capacity(dim);
        for row in 0..dim {
            let product = dense_dot(&m.cs_matrix.entries[row], &dense);
            if product != 0.0 {
                entries.push(Entry {
                    index: row,
                    value: product,
                });
            }
        }

        self.dim = dim;
        self.entries = entries;

        Ok(())
    }

    fn sort_entries_by_index(&mut self) {
        self.entries.sort_by_key(|e| e.index);
    }

    fn binary_operation<F>(&mut self, v1: &Self, v2: &Self, op: F) -> Result<()>
    where
        F: Fn(f64, f64) -> f64,
    {
        if v1.dim != v2.dim {
            return Err(Error::DimensionMismatch);
        }

        let mut entries = Vec::with_capacity(v1.entries.len() + v2.entries.len());
        let mut i1 = 0;
        let mut i2 = 0;

        while i1 < v1.entries.len() || i2 < v2.entries.len() {
            let entry = match (v1.entries.get(i1), v2.entries.get(i2)) {
                (Some(e1), Some(e2)) => match e1.index.cmp(&e2.index) {
                    Ordering::Less => {
                        i1 += 1;
                        e1.clone()
                    }
                    Ordering::Greater => {
                        i2 += 1;
                        Entry {
                            index: e2.index,
                            value: op(0.0, e2.value),
                        }
                    }
                    Ordering::Equal => {
                        i1 += 1;
                        i2 += 1;
                        Entry {
                            index: e1.index,
                            value: op(e1.value, e2.value),
                        }
                    }
                },
                (Some(e1), None) => {
                    i1 += 1;
                    e1.clone()
                }
                (None, Some(e2)) => {
                    i2 += 1;
                    Entry {
                        index: e2.index,
                        value: op(0.0, e2.value),
                    }
                }
                (None, None) => break,
            };
            entries.push(entry);
        }

        self.dim = v1.dim;
        self.entries = entries;

        Ok(())
    }

    fn scale_in_place(&mut self, a: f64) {
        self.entries.iter_mut().for_each(|e| e.value *= a);
        self.entries.retain(|e| e.value != 0.0);
    }
}

fn dense_dot(row: &[Entry], dense: &[f64]) -> f64 {
    let mut summer = KBNSummer::new();
    for e in row {
        let x = dense[e.index];
        if x != 0.0 {
            summer.add(e.value * x);
        }
    }
    summer.sum()
}

pub fn vec_dot(v1: &Vector, v2: &Vector) -> f64 {
    let mut i2_iter = v2.entries.iter();
    let mut i2_opt = i2_iter.next();
    let mut summer = KBNSummer::new();

    for e1 in &v1.entries {
        while let Some(e2) = i2_opt {
            if e2.index > e1.index {
                break;
            }
            if e2.index == e1.index {
                summer.add(e1.value * e2.value);
            }
            i2_opt = i2_iter.next();
        }
    }

    summer.sum()
}
