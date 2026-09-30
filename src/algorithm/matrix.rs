// Compressed sparse rows: `rows[i]` holds the non-zero entries of row i, sorted by column.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Entry {
    pub(crate) index: usize,
    pub(crate) value: f64,
}

impl Entry {
    pub(crate) fn new(index: usize, value: f64) -> Self {
        Entry { index, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CsrMatrix {
    pub(crate) rows: Vec<Vec<Entry>>,
}

impl CsrMatrix {
    // Sorts each row by column. Repeated columns keep the last value in input order and
    // zero values are dropped.
    pub(crate) fn from_rows(mut rows: Vec<Vec<Entry>>) -> Self {
        for row in &mut rows {
            // stable sort keeps input order among duplicates, so the last one wins
            row.sort_by_key(|e| e.index);
            dedup_keep_last(row);
            row.retain(|e| e.value != 0.0);
        }
        CsrMatrix { rows }
    }

    // Column lists come out sorted by row, which fixes the summation order of the
    // matrix-vector product.
    pub(crate) fn transpose(&self) -> CsrMatrix {
        let mut counts = vec![0usize; self.rows.len()];
        for row in &self.rows {
            for entry in row {
                counts[entry.index] += 1;
            }
        }
        let mut rows: Vec<Vec<Entry>> = counts.into_iter().map(Vec::with_capacity).collect();
        for (i, row) in self.rows.iter().enumerate() {
            for entry in row {
                rows[entry.index].push(Entry::new(i, entry.value));
            }
        }
        CsrMatrix { rows }
    }
}

fn dedup_keep_last(row: &mut Vec<Entry>) {
    if row.len() < 2 {
        return;
    }
    let mut write = 0;
    for read in 1..row.len() {
        if row[read].index == row[write].index {
            row[write].value = row[read].value;
        } else {
            write += 1;
            row[write] = row[read];
        }
    }
    row.truncate(write + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(index: usize, value: f64) -> Entry {
        Entry::new(index, value)
    }

    #[test]
    fn from_rows_sorts_dedups_and_drops_zeros() {
        let m = CsrMatrix::from_rows(vec![
            vec![e(1, 1.0), e(0, 3.0), e(1, 5.0), e(2, 0.0)],
            vec![e(1, 2.0), e(0, 4.0), e(0, 0.0)],
        ]);
        assert_eq!(m.rows, vec![vec![e(0, 3.0), e(1, 5.0)], vec![e(1, 2.0)]]);
    }

    #[test]
    fn transpose_round_trips() {
        let m = CsrMatrix::from_rows(vec![
            vec![e(0, 100.0), e(1, 200.0), e(2, 300.0)],
            vec![e(1, 400.0), e(3, 500.0)],
            vec![],
            vec![e(0, 600.0), e(1, 700.0), e(2, 800.0), e(3, 900.0)],
        ]);
        let t = m.transpose();
        assert_eq!(
            t.rows,
            vec![
                vec![e(0, 100.0), e(3, 600.0)],
                vec![e(0, 200.0), e(1, 400.0), e(3, 700.0)],
                vec![e(0, 300.0), e(3, 800.0)],
                vec![e(1, 500.0), e(3, 900.0)],
            ]
        );
        assert_eq!(t.transpose(), m);
    }
}
