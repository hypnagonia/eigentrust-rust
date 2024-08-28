use ndarray::{Array2, Array1};
use wasm_bindgen::prelude::*;
use std::error::Error;

#[derive(Debug)]
pub enum TrustError {
    DimensionMismatch,
    InvalidTrustLevel,
    CsvParseError(String),
}

impl std::fmt::Display for TrustError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustError::DimensionMismatch => write!(f, "Dimension mismatch"),
            TrustError::InvalidTrustLevel => write!(f, "Invalid trust level"),
            TrustError::CsvParseError(err) => write!(f, "CSV parse error: {}", err),
        }
    }
}

impl Error for TrustError {}

pub fn canonicalize_local_trust(local_trust: &mut Array2<f64>, pre_trust: Option<Array1<f64>>) -> Result<(), JsValue> {
    let n = local_trust.shape()[0];
    if let Some(ref pre_trust_vec) = pre_trust {
        if pre_trust_vec.len() != n {
            return Err(JsValue::from_str(&TrustError::DimensionMismatch.to_string()));
        }
    }

    for i in 0..n {
        let mut row_sum: f64 = local_trust.row(i).sum();
        if row_sum == 0.0 {
            if let Some(ref pre_trust_vec) = pre_trust {
                for j in 0..n {
                    local_trust[[i, j]] = pre_trust_vec[j];
                }
            }
        } else {
            for j in 0..n {
                local_trust[[i, j]] /= row_sum;
            }
        }
    }

    Ok(())
}

pub fn extract_distrust(local_trust: &mut Array2<f64>) -> Result<Array2<f64>, JsValue> {
    let n = local_trust.shape()[0];
    let mut distrust = Array2::<f64>::zeros((n, n));

    for truster in 0..n {
        for to in 0..n {
            let entry = local_trust[[truster, to]];
            if entry < 0.0 {
                distrust[[truster, to]] = -entry;
                local_trust[[truster, to]] = 0.0;
            }
        }
    }

    Ok(distrust)
}

// Helper function to parse CSV data (this is a placeholder for the actual CSV reading)
fn parse_csv_line(line: &str) -> Result<(usize, usize, f64), TrustError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() < 2 {
        return Err(TrustError::CsvParseError("Too few fields".to_string()));
    }
    let from = fields[0].parse::<usize>().map_err(|_| TrustError::InvalidTrustLevel)?;
    let to = fields[1].parse::<usize>().map_err(|_| TrustError::InvalidTrustLevel)?;
    let level = if fields.len() >= 3 {
        fields[2].parse::<f64>().map_err(|_| TrustError::InvalidTrustLevel)?
    } else {
        1.0
    };
    Ok((from, to, level))
}

// ReadLocalTrustFromCsv function in Rust
pub fn read_local_trust_from_csv(csv_data: &str, peer_indices: JsValue) -> Result<Array2<f64>, JsValue> {
    let peer_indices_map: std::collections::HashMap<String, usize> = peer_indices.into_serde().map_err(|_| JsValue::from_str("Failed to parse peer indices"))?;

    let mut entries: Vec<(usize, usize, f64)> = Vec::new();
    let mut max_index = 0;

    for (count, line) in csv_data.lines().enumerate() {
        let (from, to, level) = parse_csv_line(line).map_err(|e| JsValue::from_str(&format!("Cannot parse local trust CSV record #{}: {}", count + 1, e)))?;

        if max_index < from {
            max_index = from;
        }
        if max_index < to {
            max_index = to;
        }

        entries.push((from, to, level));
    }

    let dim = max_index + 1;
    let mut local_trust = Array2::<f64>::zeros((dim, dim));

    for (from, to, level) in entries {
        local_trust[[from, to]] = level;
    }

    Ok(local_trust)
}


#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_extract_distrust() {
        struct TestCase {
            name: &'static str,
            local_trust: Array2<f64>,
            expected_trust: Array2<f64>,
            expected_distrust: Array2<f64>,
        }

        let test_cases = vec![
            TestCase {
                name: "test1",
                local_trust: array![
                    [100.0, -50.0, -50.0],
                    [0.0, 0.0, 0.0],
                    [-100.0, 0.0, 0.0],
                ],
                expected_trust: array![
                    [100.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                expected_distrust: array![
                    [0.0, 50.0, 50.0],
                    [0.0, 0.0, 0.0],
                    [100.0, 0.0, 0.0],
                ],
            },
        ];

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
