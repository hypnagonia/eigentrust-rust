use super::input::{for_each_record, parse_weight};
use super::util::PeersMap;
use crate::error::{Error, Input, RecordError, Result};
use crate::sparse::entry::Entry;
use crate::sparse::matrix::CSRMatrix;
use crate::sparse::vector::Vector;

// Scales every row to sum to one. Rows without trust (dangling peers) get the pre-trust
// distribution, so their share flows back to the seeds.
pub fn canonicalize_local_trust(
    local_trust: &mut CSRMatrix,
    pre_trust: Option<&Vector>,
) -> Result<()> {
    let n = local_trust.dims().0;

    if let Some(pre_trust) = pre_trust {
        if pre_trust.entries.len() > n {
            return Err(Error::DimensionMismatch);
        }
    }

    for row in local_trust.cs_matrix.entries.iter_mut() {
        let row_sum: f64 = row.iter().map(|entry| entry.value).sum();
        if row_sum == 0.0 {
            if let Some(pre_trust) = pre_trust {
                row.clone_from(&pre_trust.entries);
            }
        } else {
            for entry in row.iter_mut() {
                entry.value /= row_sum;
            }
        }
    }

    Ok(())
}

// Splits negative entries off into a separate distrust matrix (as positive values).
// Not used by calculate_from_csv: negative weights are rejected while parsing until the
// effect of distrust on the scores is defined.
pub fn extract_distrust(local_trust: &mut CSRMatrix) -> CSRMatrix {
    let n = local_trust.dims().0;
    let mut distrust = CSRMatrix::new(n, n, vec![]);

    for (truster, row) in local_trust.cs_matrix.entries.iter_mut().enumerate() {
        let mut distrust_row = Vec::new();
        row.retain(|entry| {
            if entry.value >= 0.0 {
                true
            } else {
                distrust_row.push(Entry::new(entry.index, -entry.value));
                false
            }
        });
        distrust.set_row_vector(truster, Vector::new(n, distrust_row));
    }

    distrust
}

// Reads `from,to[,weight]` records into a square CSR matrix, one row per truster.
// Peers get indices in order of first appearance. A repeated `from,to` pair keeps the
// last weight.
pub fn read_local_trust_from_csv(csv_data: &str) -> Result<(CSRMatrix, PeersMap)> {
    // rows are filled directly while parsing; the matrix grows as new peers appear
    let mut rows: Vec<Vec<Entry>> = Vec::new();
    let mut peers = PeersMap::new();

    for_each_record(csv_data, Input::LocalTrust, 2, |line, fields| {
        let record_error = |error| Error::Record {
            input: Input::LocalTrust,
            line,
            error,
        };
        let (from, to) = match fields {
            [from, to, ..] => (*from, *to),
            _ => return Err(record_error(RecordError::TooFewFields)),
        };
        let level = parse_weight(fields.get(2)).map_err(record_error)?;
        let from = peers.insert_or_get(from);
        let to = peers.insert_or_get(to);
        if from >= rows.len() {
            rows.resize_with(from + 1, Vec::new);
        }
        rows[from].push(Entry::new(to, level));
        Ok(())
    })?;

    let dim = peers.names.len();
    if dim == 0 {
        return Err(Error::EmptyLocalTrust);
    }
    rows.resize_with(dim, Vec::new);
    Ok((CSRMatrix::from_rows(dim, rows), peers))
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
            let distrust = extract_distrust(&mut local_trust);

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

    #[test]
    fn test_local_trust_rejects_bad_levels() {
        for bad in ["a,b,NaN", "a,b,inf", "a,b,-inf", "a,b,-1", "a,b,x"] {
            assert!(read_local_trust_from_csv(bad).is_err(), "{}", bad);
        }
        assert!(read_local_trust_from_csv("a,b,0\nb,a,2.5").is_ok());
    }
}
