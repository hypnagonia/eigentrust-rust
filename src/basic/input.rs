// CSV input: a real CSV reader (quotes, embedded commas, escaped quotes), an optional
// header row, surrounding whitespace, CRLF and a UTF-8 BOM.

use crate::error::{Error, Input, RecordError, Result};
use csv::{ByteRecord, ReaderBuilder};

const HEADER_NAMES: &[&str] = &[
    "i", "j", "v", "from", "to", "value", "weight", "trust", "level", "peer", "id", "score",
    "source", "target", "src", "dst", "truster", "trustee",
];

// Surrounding whitespace and quotes the CSV reader keeps, e.g. in ` "alice" `.
pub fn clean_field(field: &str) -> &str {
    let field = field.trim();
    field
        .strip_prefix('"')
        .and_then(|f| f.strip_suffix('"'))
        .unwrap_or(field)
        .trim()
}

// A first record is a header when its value column is not a number, or, when the value
// column is omitted (implicit weight 1), when every field is a well-known header name.
// `value_column` is 2 for local trust and 1 for pre-trust.
pub fn is_header(fields: &[&str], value_column: usize) -> bool {
    match fields.get(value_column) {
        Some(value) => value.parse::<f64>().is_err(),
        None => fields
            .iter()
            .all(|f| HEADER_NAMES.contains(&f.to_ascii_lowercase().as_str())),
    }
}

// Calls `f(line, fields)` for every non-empty record after the optional header.
pub fn for_each_record<F>(data: &str, input: Input, value_column: usize, mut f: F) -> Result<()>
where
    F: FnMut(u64, &[&str]) -> Result<()>,
{
    let data = data.trim_start_matches('\u{feff}');
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());

    // the csv crate reports a record's position before any blank lines it skipped and
    // does not count those lines, so line numbers come from byte offsets
    let bytes = data.as_bytes();
    let mut counted = (0usize, 1u64);
    let mut line_at = |byte: u64| {
        let mut byte = (byte as usize).min(bytes.len());
        while byte < bytes.len() && (bytes[byte] == b'\n' || bytes[byte] == b'\r') {
            byte += 1;
        }
        if byte >= counted.0 {
            counted.1 += bytes[counted.0..byte]
                .iter()
                .filter(|&&b| b == b'\n')
                .count() as u64;
            counted.0 = byte;
        }
        counted.1
    };

    let mut record = ByteRecord::new();
    let mut first = true;
    loop {
        let more = match reader.read_byte_record(&mut record) {
            Ok(more) => more,
            Err(e) => {
                return Err(Error::Record {
                    input,
                    line: e.position().map_or(0, |p| line_at(p.byte())),
                    error: RecordError::Malformed(e.to_string()),
                })
            }
        };
        if !more {
            return Ok(());
        }
        let line = record.position().map_or(0, |p| line_at(p.byte()));

        // at most 3 columns are used, extra columns are ignored
        let mut buf = [""; 3];
        let mut n = 0;
        for field in record.iter().take(buf.len()) {
            // the input is a &str, but a quoted field could still split a code point
            let field = std::str::from_utf8(field).map_err(|e| Error::Record {
                input,
                line,
                error: RecordError::Malformed(e.to_string()),
            })?;
            buf[n] = clean_field(field);
            n += 1;
        }
        let fields = &buf[..n];
        if fields.iter().all(|f| f.is_empty()) {
            continue;
        }
        if first {
            first = false;
            if is_header(fields, value_column) {
                continue;
            }
        }
        f(line, fields)?;
    }
}

// A trust weight: finite and non-negative, 1 when omitted.
pub fn parse_weight(field: Option<&&str>) -> std::result::Result<f64, RecordError> {
    let Some(&field) = field else {
        return Ok(1.0);
    };
    let weight = field
        .parse::<f64>()
        .map_err(|_| RecordError::InvalidWeight(field.to_string()))?;
    if !weight.is_finite() {
        return Err(RecordError::NonFiniteWeight(field.to_string()));
    }
    if weight < 0.0 {
        return Err(RecordError::NegativeWeight(field.to_string()));
    }
    Ok(weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(data: &str, value_column: usize) -> Result<Vec<(u64, Vec<String>)>> {
        let mut out = vec![];
        for_each_record(data, Input::LocalTrust, value_column, |line, f| {
            out.push((line, f.iter().map(|s| s.to_string()).collect()));
            Ok(())
        })?;
        Ok(out)
    }

    fn rows(data: &str, value_column: usize) -> Vec<Vec<String>> {
        records(data, value_column)
            .unwrap()
            .into_iter()
            .map(|(_, f)| f)
            .collect()
    }

    #[test]
    fn test_headers() {
        assert_eq!(rows("i,j,v\na,b,1\n", 2), [["a", "b", "1"]]);
        assert_eq!(rows("i,j,v\r\na,b,1\r\n", 2), [["a", "b", "1"]]);
        assert_eq!(rows("\u{feff}from,to,weight\na,b,1", 2), [["a", "b", "1"]]);
        // headerless two-column local trust keeps its first edge
        assert_eq!(rows("a,b\nb,c", 2), [["a", "b"], ["b", "c"]]);
        assert_eq!(rows("i,j\na,b", 2), [["a", "b"]]);
        assert_eq!(rows("i,v\nalice,1", 1), [["alice", "1"]]);
        assert_eq!(rows("alice\nbob", 1), [["alice"], ["bob"]]);
        assert_eq!(rows("peer\nalice", 1), [["alice"]]);
        assert!(rows("", 1).is_empty());
    }

    #[test]
    fn test_real_csv() {
        // quoted fields with commas and escaped quotes, blank lines, spaces
        let data = "\"Smith, J\",\"O\"\"Neil\",2\n\n  a , b , 3 \n \"c\" , \"d\" ,1\n";
        assert_eq!(
            rows(data, 2),
            [
                ["Smith, J", "O\"Neil", "2"],
                ["a", "b", "3"],
                ["c", "d", "1"]
            ]
        );
    }

    #[test]
    fn test_line_numbers() {
        let lines: Vec<u64> = records("i,j,v\na,b,1\n\nc,d,2\r\n\r\n\r\ne,f", 2)
            .unwrap()
            .into_iter()
            .map(|(l, _)| l)
            .collect();
        assert_eq!(lines, [2, 4, 7]);
    }

    #[test]
    fn test_parse_weight() {
        assert_eq!(parse_weight(None), Ok(1.0));
        assert_eq!(parse_weight(Some(&"2.5")), Ok(2.5));
        assert!(matches!(
            parse_weight(Some(&"x")),
            Err(RecordError::InvalidWeight(_))
        ));
        for bad in ["NaN", "inf", "-inf"] {
            assert!(matches!(
                parse_weight(Some(&bad)),
                Err(RecordError::NonFiniteWeight(_))
            ));
        }
        assert!(matches!(
            parse_weight(Some(&"-1")),
            Err(RecordError::NegativeWeight(_))
        ));
    }

    #[test]
    fn test_clean_field() {
        assert_eq!(clean_field("  alice "), "alice");
        assert_eq!(clean_field("\"alice\""), "alice");
        assert_eq!(clean_field(" \" 0.5 \" "), "0.5");
        assert_eq!(clean_field("\""), "\"");
    }
}
