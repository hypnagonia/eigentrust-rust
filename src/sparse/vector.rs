use std::cmp::Ordering;
use std::f64;
use std::sync::{ Arc, Mutex };
use std::thread;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Debug)]
pub struct Entry {
    pub index: usize,
    pub value: f64,
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            value: self.value,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Vector {
    dim: usize,
    entries: Vec<Entry>,
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
        self.entries
            .iter()
            .map(|e| e.value)
            .sum()
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
                (Some(e1_first), Some(e2_first)) =>
                    match e1_first.index.cmp(&e2_first.index) {
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
                    }
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
                (Some(e1_first), Some(e2_first)) =>
                    match e1_first.index.cmp(&e2_first.index) {
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
                    }
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
    
        // First pass: Scale the values and count zeros
        let mut non_zero_entries = Vec::with_capacity(self.entries.len());
        for entry in &mut self.entries {
            entry.value *= a;
            if entry.value != 0.0 {
                non_zero_entries.push(entry.clone());
            }
        }
    
        // Update the entries with non-zero values
        self.entries = non_zero_entries;
    }

    pub fn norm2(&self) -> f64 {
        self.entries
            .iter()
            .map(|e| e.value * e.value)
            .sum::<f64>()
            .sqrt()
    }

    fn sort_entries_by_index(&mut self) {
        self.entries.sort_by_key(|e| e.index);
    }
}

pub fn vec_dot(v1: &Vector, v2: &Vector) -> f64 {
    let mut sum = 0.0;
    let mut i2 = 0;
    let n2 = v2.entries.len();
    if n2 == 0 {
        return 0.0;
    }

    for e1 in &v1.entries {
        while i2 < n2 && v2.entries[i2].index <= e1.index {
            if e1.index == v2.entries[i2].index {
                sum += e1.value * v2.entries[i2].value;
            }
            i2 += 1;
        }
    }

    sum
}

#[derive(Clone, PartialEq, Debug)]
pub struct Matrix {
    // Matrix fields and methods should be implemented here.
    // Placeholder for the matrix implementation.
}

impl Matrix {
    fn dim(&self) -> Result<usize, String> {
        // Implement matrix dimension logic here.
        Ok(0) // Placeholder
    }

    fn row_vector(&self, row: usize) -> Vector {
        // Implement row vector extraction logic here.
        Vector::new(0, Vec::new()) // Placeholder
    }
}

// todo
pub fn mul_vec(ctx: JsValue, m: Matrix, v1: Vector) -> Result<Vector, String> {
    let dim = m.dim()?; // Get the dimension of the matrix.
    if dim != v1.dim {
        return Err("Dimension mismatch".to_string());
    }

    // Use `Arc` to share the data between threads safely.
    let jobs = Arc::new(Mutex::new((0..dim).collect::<Vec<usize>>()));
    let entries = Arc::new(Mutex::new(Vec::with_capacity(dim)));
    let mut handles = vec![];

    for _ in 0..32 {
        let jobs = Arc::clone(&jobs);
        let entries = Arc::clone(&entries);
        let m_cloned = m.clone(); // Now `m` is owned, and we clone the owned data.
        let v1_cloned = v1.clone(); // Similarly, `v1` is owned and cloned.

        let handle = thread::spawn(move || {
            while let Some(row) = {
                let mut jobs = jobs.lock().unwrap();
                jobs.pop()
            } {
                // Compute the dot product for the current row.
                let product = vec_dot(&m_cloned.row_vector(row), &v1_cloned);
                let mut entries = entries.lock().unwrap();
                if product != 0.0 {
                    entries.push(Entry { index: row, value: product });
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete.
    for handle in handles {
        handle.join().unwrap();
    }

    // Collect and sort the results.
    let mut sorted_entries = Arc::try_unwrap(entries).unwrap().into_inner().unwrap();
    sorted_entries.sort_by_key(|e| e.index);

    Ok(Vector::new(dim, sorted_entries))
}