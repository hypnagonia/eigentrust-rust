use wasm_bindgen::prelude::*;
use std::cmp::Ordering;
use serde::Serialize;

// Define the Entry struct
#[derive(Debug, Clone, PartialEq, Serialize)] 
pub struct Entry {
    pub index: usize,
    pub value: f64,
}

impl Entry {
    pub fn new(index: usize, value: f64) -> Entry {
        Entry { index, value }
    }
}

// Define the CooEntry struct
#[derive(Debug, Clone, PartialEq, Serialize)] 
pub struct CooEntry {
    pub row: usize,
    pub column: usize,
    pub value: f64,
}

impl CooEntry {
    pub fn new(row: usize, column: usize, value: f64) -> CooEntry {
        CooEntry { row, column, value }
    }
}

// Implement sorting logic for CSR entries
pub struct CSREntriesSort(Vec<CooEntry>);

impl CSREntriesSort {
    pub fn new(entries: Vec<CooEntry>) -> Self {
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| match a.row.cmp(&b.row) {
            Ordering::Equal => a.column.cmp(&b.column),
            other => other,
        });
        CSREntriesSort(sorted_entries)
    }

    pub fn entries(&self) -> Vec<CooEntry> {
        self.0.clone()
    }
}

// Implement sorting logic for CSC entries
pub struct CSCEntriesSort(Vec<CooEntry>);

impl CSCEntriesSort {
    pub fn new(entries: Vec<CooEntry>) -> Self {
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| match a.column.cmp(&b.column) {
            Ordering::Equal => a.row.cmp(&b.row),
            other => other,
        });
        CSCEntriesSort(sorted_entries)
    }

    pub fn entries(&self) -> Vec<CooEntry> {
        self.0.clone()
    }
}

// Implement sorting by index for Entries
pub fn sort_entries_by_index(entries: &mut [Entry]) {
    entries.sort_by(|a, b| a.index.cmp(&b.index));
}

// Implement sorting by value for Entries
pub fn sort_entries_by_value(entries: &mut [Entry]) {
    entries.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_entries_sort_len() {
        let tests = vec![
            (
                "Normal",
                vec![
                    CooEntry::new(3, 1, 7.0),
                    CooEntry::new(1, 0, 4.0),
                    CooEntry::new(2, 8, 0.0),
                    CooEntry::new(5, 0, 0.0),
                ],
                4,
            ),
            ("Empty", vec![], 0),
        ];

        for (name, mut entries, expected_len) in tests {
            let len = entries.len();
            assert_eq!(len, expected_len, "{}: len = {}, want {}", name, len, expected_len);
        }
    }

    #[test]
    fn test_csr_entries_sort_swap() {
        let mut entries = vec![
            CooEntry::new(3, 1, 7.0),
            CooEntry::new(1, 0, 4.0),
            CooEntry::new(2, 8, 0.0),
            CooEntry::new(5, 0, 0.0),
        ];

        entries.swap(1, 2);

        let expected = vec![
            CooEntry::new(3, 1, 7.0),
            CooEntry::new(2, 8, 0.0),
            CooEntry::new(1, 0, 4.0),
            CooEntry::new(5, 0, 0.0),
        ];

        assert_eq!(entries, expected);
    }

    #[test]
    fn test_csr_entries_sort_less() {
        let tests = vec![
            ("xr<yr,xc<yc", CooEntry::new(0, 0, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr<yr,xc=yc", CooEntry::new(0, 1, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr<yr,xc>yc", CooEntry::new(0, 2, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr=yr,xc<yc", CooEntry::new(1, 0, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr=yr,xc=yc", CooEntry::new(1, 1, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr=yr,xc>yc", CooEntry::new(1, 2, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr>yr,xc<yc", CooEntry::new(2, 0, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr>yr,xc=yc", CooEntry::new(2, 1, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr>yr,xc>yc", CooEntry::new(2, 2, 0.0), CooEntry::new(1, 1, 0.0), false),
        ];

        for (name, x, y, expected) in tests {
            let entries = vec![x.clone(), y.clone()];
            let result = entries[0].row < entries[1].row || (entries[0].row == entries[1].row && entries[0].column < entries[1].column);
            assert_eq!(result, expected, "{}: got = {}, want {}", name, result, expected);
        }
    }

    #[test]
    fn test_csc_entries_sort_len() {
        let tests = vec![
            (
                "Normal",
                vec![
                    CooEntry::new(3, 1, 7.0),
                    CooEntry::new(1, 0, 4.0),
                    CooEntry::new(2, 8, 0.0),
                    CooEntry::new(5, 0, 0.0),
                ],
                4,
            ),
            ("Empty", vec![], 0),
        ];

        for (name, mut entries, expected_len) in tests {
            let len = entries.len();
            assert_eq!(len, expected_len, "{}: len = {}, want {}", name, len, expected_len);
        }
    }

    #[test]
    fn test_csc_entries_sort_swap() {
        let mut entries = vec![
            CooEntry::new(3, 1, 7.0),
            CooEntry::new(1, 0, 4.0),
            CooEntry::new(2, 8, 0.0),
            CooEntry::new(5, 0, 0.0),
        ];

        entries.swap(1, 2);

        let expected = vec![
            CooEntry::new(3, 1, 7.0),
            CooEntry::new(2, 8, 0.0),
            CooEntry::new(1, 0, 4.0),
            CooEntry::new(5, 0, 0.0),
        ];

        assert_eq!(entries, expected);
    }

    #[test]
    fn test_csc_entries_sort_less() {
        let tests = vec![
            ("xr<yr,xc<yc", CooEntry::new(0, 0, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr=yr,xc<yc", CooEntry::new(1, 0, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr>yr,xc<yc", CooEntry::new(2, 0, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr<yr,xc=yc", CooEntry::new(0, 1, 0.0), CooEntry::new(1, 1, 0.0), true),
            ("xr=yr,xc=yc", CooEntry::new(1, 1, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr>yr,xc=yc", CooEntry::new(2, 1, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr<yr,xc>yc", CooEntry::new(0, 2, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr=yr,xc>yc", CooEntry::new(1, 2, 0.0), CooEntry::new(1, 1, 0.0), false),
            ("xr>yr,xc>yc", CooEntry::new(2, 2, 0.0), CooEntry::new(1, 1, 0.0), false),
        ];

        for (name, x, y, expected) in tests {
            let entries = vec![x.clone(), y.clone()];
            let result = entries[0].column < entries[1].column || (entries[0].column == entries[1].column && entries[0].row < entries[1].row);
            assert_eq!(result, expected, "{}: got = {}, want {}", name, result, expected);
        }
    }
}
