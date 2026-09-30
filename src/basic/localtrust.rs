use super::util::{clean_field, PeersMap};
use crate::sparse::entry::Entry;
use crate::sparse::matrix::CSRMatrix;
use crate::sparse::vector::Vector;

pub fn canonicalize_local_trust(
    local_trust: &mut CSRMatrix,
    pre_trust: Option<Vector>,
) -> Result<(), String> {
    let n = local_trust.dims().0;

    if let Some(ref pre_trust_vec) = pre_trust {
        // if pre_trust_vec.entries.len() != n {
        if pre_trust_vec.entries.len() > n {
            return Err("Dimension mismatch".to_string());
        }
    }

    for i in 0..n {
        let mut in_row = local_trust.row_vector(i);
        let row_sum: f64 = in_row.entries.iter().map(|entry| entry.value).sum();

        if row_sum == 0.0 {
            if let Some(ref pre_trust_vec) = pre_trust {
                local_trust.set_row_vector(i, Vector::new(n, pre_trust_vec.entries.clone()));
            }
        } else {
            for entry in &mut in_row.entries {
                entry.value /= row_sum;
            }
            local_trust.set_row_vector(i, in_row);
        }
    }

    Ok(())
}

pub fn extract_distrust(local_trust: &mut CSRMatrix) -> Result<CSRMatrix, String> {
    let n = local_trust.dims().0;
    let mut distrust = CSRMatrix::new(n, n, vec![]);

    for truster in 0..n {
        let mut trust_row = local_trust.row_vector(truster);
        let mut distrust_row = Vec::new();

        trust_row.entries.retain(|entry| {
            if entry.value >= 0.0 {
                true
            } else {
                distrust_row.push(Entry {
                    index: entry.index,
                    value: -entry.value,
                });
                false
            }
        });

        local_trust.set_row_vector(truster, trust_row);
        distrust.set_row_vector(truster, Vector::new(n, distrust_row));
    }

    Ok(distrust)
}

fn parse_csv_line(line: &str, peer_indices: &mut PeersMap) -> Result<(usize, usize, f64), String> {
    let mut fields = line.split(',').map(clean_field);

    let (from, to) = match (fields.next(), fields.next()) {
        (Some(from), Some(to)) => (from, to),
        _ => return Err("Too few fields".to_string()),
    };
    let from = peer_indices.insert_or_get(from);
    let to = peer_indices.insert_or_get(to);
    let level = match fields.next() {
        Some(level) => level.parse::<f64>().map_err(|_| "Invalid trust level")?,
        None => 1.0,
    };
    Ok((from, to, level))
}

// todo move csv logic out of this scope, cooentry
pub fn read_local_trust_from_csv(csv_data: &str) -> Result<(CSRMatrix, PeersMap), String> {
    // rows are filled directly while parsing; the matrix grows as new peers appear
    let mut rows: Vec<Vec<Entry>> = Vec::new();
    let mut peer_indices = PeersMap::new();

    for (count, line) in csv_data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (from, to, level) = parse_csv_line(line, &mut peer_indices).map_err(|e| {
            format!(
                "Cannot parse local trust CSV record #{}: {:?} {:?}",
                count + 1,
                e,
                line
            )
        })?;
        if from >= rows.len() {
            rows.resize_with(from + 1, Vec::new);
        }
        rows[from].push(Entry::new(to, level));
    }

    let dim = peer_indices.names.len();
    if dim == 0 {
        return Err("Local trust is empty".to_string());
    }
    rows.resize_with(dim, Vec::new);
    Ok((CSRMatrix::from_rows(dim, rows), peer_indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_distrust() {
        struct TestCase {
            name: &'static str,
            local_trust: CSRMatrix,
            expected_trust: CSRMatrix,
            expected_distrust: CSRMatrix,
        }

        let test_cases = vec![TestCase {
            name: "test1",
            local_trust: CSRMatrix::new(
                3,
                3,
                vec![(0, 0, 100.0), (0, 1, -50.0), (0, 2, -50.0), (2, 0, -100.0)],
            ),
            expected_trust: CSRMatrix::new(3, 3, vec![(0, 0, 100.0)]),
            expected_distrust: CSRMatrix::new(
                3,
                3,
                vec![(0, 1, 50.0), (0, 2, 50.0), (2, 0, 100.0)],
            ),
        }];

        for test in test_cases {
            let mut local_trust = test.local_trust.clone();
            let distrust = extract_distrust(&mut local_trust).expect("Failed to extract distrust");

            assert_eq!(
                local_trust, test.expected_trust,
                "{}: local trust does not match expected value",
                test.name
            );
            assert_eq!(
                distrust, test.expected_distrust,
                "{}: distrust does not match expected value",
                test.name
            );
        }
    }
}
