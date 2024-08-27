use wasm_bindgen::prelude::*;
use std::cmp::Ordering;
use std::slice;

// Entry struct similar to Go's sparse matrix entry
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub index: usize,
    pub value: f64,
}

impl Entry {
    pub fn new(index: usize, value: f64) -> Entry {
        Entry { index, value }
    }
}

// Compressed Sparse Matrix (CSMatrix)
#[derive(Debug, Clone, PartialEq)]
pub struct CSMatrix {
    major_dim: usize,
    minor_dim: usize,
    entries: Vec<Vec<Entry>>,
}

impl CSMatrix {
    pub fn new(major_dim: usize, minor_dim: usize, entries: Vec<Vec<Entry>>) -> CSMatrix {
        CSMatrix {
            major_dim,
            minor_dim,
            entries,
        }
    }

    pub fn reset(&mut self) {
        self.major_dim = 0;
        self.minor_dim = 0;
        self.entries.clear();
    }

    pub fn dim(&self) -> Result<usize, String> {
        if self.major_dim != self.minor_dim {
            Err("Dimension mismatch".to_string())
        } else {
            Ok(self.major_dim)
        }
    }

    pub fn set_major_dim(&mut self, dim: usize) {
        if self.entries.capacity() < dim {
            self.entries.resize(dim, Vec::new());
        }
        self.major_dim = dim;
    }

    pub fn set_minor_dim(&mut self, dim: usize) {
        self.minor_dim = dim;
        for entry in &mut self.entries {
            entry.retain(|e| e.index < dim);
        }
    }

    pub fn nnz(&self) -> usize {
        self.entries
            .iter()
            .map(|row| row.len())
            .sum()
    }

    pub fn transpose(&self) -> Result<CSMatrix, String> {
        let mut nnzs = vec![0; self.minor_dim];
        for row in &self.entries {
            for e in row {
                nnzs[e.index] += 1;
            }
        }

        let mut transposed_entries = vec![Vec::new(); self.minor_dim];
        for (col, &nnz) in nnzs.iter().enumerate() {
            if nnz > 0 {
                transposed_entries[col] = Vec::with_capacity(nnz);
            }
        }

        for (row, row_entries) in self.entries.iter().enumerate() {
            for e in row_entries {
                transposed_entries[e.index].push(Entry::new(row, e.value));
            }
        }

        Ok(CSMatrix {
            major_dim: self.minor_dim,
            minor_dim: self.major_dim,
            entries: transposed_entries,
        })
    }

    pub fn merge(&mut self, other: &mut CSMatrix) {
        self.set_major_dim(usize::max(self.major_dim, other.major_dim));
        self.set_minor_dim(usize::max(self.minor_dim, other.minor_dim));

        for i in 0..other.major_dim {
            let merged = merge_span(&self.entries[i], &other.entries[i]);
            self.entries[i] = merged;
        }
        other.reset();
    }
}

fn merge_span(s1: &[Entry], s2: &[Entry]) -> Vec<Entry> {
    let mut result = Vec::with_capacity(s1.len() + s2.len());
    let (mut i1, mut i2) = (0, 0);

    while i1 < s1.len() || i2 < s2.len() {
        if i1 < s1.len() && (i2 >= s2.len() || s1[i1].index < s2[i2].index) {
            result.push(s1[i1].clone());
            i1 += 1;
        } else if i2 < s2.len() && (i1 >= s1.len() || s1[i1].index > s2[i2].index) {
            result.push(s2[i2].clone());
            i2 += 1;
        } else {
            result.push(s2[i2].clone());
            i1 += 1;
            i2 += 1;
        }
    }

    result.shrink_to_fit();
    result
}

// Compressed Sparse Row (CSR) Matrix
#[derive(Debug, Clone)]
pub struct CSRMatrix {
    cs_matrix: CSMatrix,
}

impl CSRMatrix {
    pub fn new(rows: usize, cols: usize, entries: Vec<Entry>) -> CSRMatrix {
        let mut entries2: Vec<Vec<Entry>> = vec![Vec::new(); rows];

        for e in entries {
            if e.value != 0.0 {
                entries2[e.index].push(e);
            }
        }

        for row in entries2.iter_mut() {
            row.sort_by(|a, b| a.index.cmp(&b.index));
        }

        CSRMatrix {
            cs_matrix: CSMatrix::new(rows, cols, entries2),
        }
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.cs_matrix.major_dim, self.cs_matrix.minor_dim)
    }

    pub fn set_dim(&mut self, rows: usize, cols: usize) {
        self.cs_matrix.set_major_dim(rows);
        self.cs_matrix.set_minor_dim(cols);
    }

    pub fn transpose(&self) -> Result<CSRMatrix, String> {
        self.cs_matrix.transpose().map(|transposed| CSRMatrix {
            cs_matrix: transposed,
        })
    }
}

// Compressed Sparse Column (CSC) Matrix
#[derive(Debug, Clone, PartialEq)]
pub struct CSCMatrix {
    cs_matrix: CSMatrix,
}

impl CSCMatrix {
    pub fn new(cols: usize, rows: usize, entries: Vec<Entry>) -> CSCMatrix {
        let mut entries2: Vec<Vec<Entry>> = vec![Vec::new(); cols];

        for e in entries {
            if e.value != 0.0 {
                entries2[e.index].push(e);
            }
        }

        for col in entries2.iter_mut() {
            col.sort_by(|a, b| a.index.cmp(&b.index));
        }

        CSCMatrix {
            cs_matrix: CSMatrix::new(cols, rows, entries2),
        }
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.cs_matrix.minor_dim, self.cs_matrix.major_dim)
    }

    pub fn transpose(&self) -> Result<CSCMatrix, String> {
        self.cs_matrix.transpose().map(|transposed| CSCMatrix {
            cs_matrix: transposed,
        })
    }
}


// NewCSRMatrix constructor function
/*
fn new_csr_matrix(rows: usize, cols: usize, entries: Vec<(i32, i32, f64)>) -> CSRMatrix {
    let mut cs_matrix = CSMatrix {
        major_dim: rows,
        minor_dim: cols,
        entries: vec![vec![]; rows],
    };

    for (row, col, value) in entries {
        if value != 0 {
            cs_matrix.entries[row].push(Entry {
                index: usize::from(col),
                value,
            });
        }
    }

    CSRMatrix { cs_matrix }
}
 */



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cs_matrix_transpose() {
        let original = CSMatrix {
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
        };

        let transposed = CSMatrix {
            major_dim: 4,
            minor_dim: 5,
            entries: vec![
                vec![Entry { index: 0, value: 100.0 }, Entry { index: 3, value: 600.0 }],
                vec![
                    Entry { index: 0, value: 200.0 },
                    Entry { index: 1, value: 400.0 },
                    Entry { index: 3, value: 700.0 }
                ],
                vec![
                    Entry { index: 0, value: 300.0 },
                    Entry { index: 3, value: 800.0 },
                    Entry { index: 4, value: 1000.0 }
                ],
                vec![Entry { index: 1, value: 500.0 }, Entry { index: 3, value: 900.0 }]
            ],
        };

        let result = original.transpose().unwrap();
        assert_eq!(result, transposed);

        let double_transpose = result.transpose().unwrap();
        assert_eq!(double_transpose, original);
    }

    #[test]
    fn test_cs_matrix_merge() {
        let mut m = CSMatrix {
            major_dim: 3,
            minor_dim: 3,
            entries: vec![
                vec![],
                vec![Entry { index: 2, value: 5.0 }],
                vec![Entry { index: 1, value: 5.0 }, Entry { index: 2, value: 5.0 }]
            ],
        };

        let mut m2 = CSMatrix {
            major_dim: 4,
            minor_dim: 4,
            entries: vec![
                vec![Entry { index: 0, value: 8.0 }, Entry { index: 2, value: 8.0 }],
                vec![Entry { index: 0, value: 8.0 }],
                vec![Entry { index: 1, value: 8.0 }, Entry { index: 3, value: 8.0 }],
                vec![Entry { index: 1, value: 8.0 }, Entry { index: 2, value: 8.0 }]
            ],
        };

        let merged = CSMatrix {
            major_dim: 4,
            minor_dim: 4,
            entries: vec![
                vec![Entry { index: 0, value: 8.0 }, Entry { index: 2, value: 8.0 }],
                vec![Entry { index: 0, value: 8.0 }, Entry { index: 2, value: 5.0 }],
                vec![
                    Entry { index: 1, value: 8.0 },
                    Entry { index: 2, value: 5.0 },
                    Entry { index: 3, value: 8.0 }
                ],
                vec![Entry { index: 1, value: 8.0 }, Entry { index: 2, value: 8.0 }]
            ],
        };

        m.merge(&mut m2);
        assert_eq!(m, merged);
    }

    // #[test]
    /*
    fn test_new_csr_matrix() {
        let entries = vec![
            (0, 0, 100.0),
            (3, 0, 600.0),
            (3, 1, 700.0),
            (1, 1, 400.0),
            (0, 1, 200.0),
            (2, 1, 0.0), // zero value should be dropped
            (1, 3, 500.0),
            (3, 3, 900.0),
            (4, 2, 1000.0),
            (0, 2, 300.0),
            (3, 2, 800.0)
        ];

        let expected = CSRMatrix {
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

        let result = new_csr_matrix(5, 4, entries);
        assert_eq!(result, expected);
    }
     */
}
