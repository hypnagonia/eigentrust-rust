//! Named peers from CSV text. Needs the `csv` feature.
//!
//! Local trust has one `from,to[,weight]` record per line, pre-trust one `peer[,weight]`
//! record. Peer names are arbitrary strings and weights default to 1.
//!
//! ```
//! use eigentrust::csv::Network;
//! use eigentrust::EigenTrustOptions;
//!
//! let network = Network::from_csv(
//!     "from,to,weight\nalice,bob,2\nbob,carol,1\ncarol,alice,1\n",
//!     "alice\n",
//! )?;
//! let result = network.eigentrust(&EigenTrustOptions::default())?;
//! let ranking = network.ranking(&result);
//! assert_eq!(ranking[0].0, "alice");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The reader accepts standard CSV: quoted fields (`"Smith, J"`), escaped quotes, an
//! optional header row, spaces around fields, CRLF line endings and a UTF-8 BOM. Blank lines
//! are skipped and columns after the weight are ignored.
//!
//! The first record is a header when its weight column is not a number, or, for records
//! without a weight column, when every field is a common header name such as `from`, `to`,
//! `peer` or `weight`. A weight that fails to parse on any later line is an error.

use crate::error::Input;
use crate::{eigentrust_with_options, EigenTrustError, EigenTrustOptions};
use crate::{PreTrust, TrustEdge, TrustScores};
use std::collections::HashMap;
use std::fmt;

/// A network with named peers, read from CSV.
///
/// Peers get indices in order of first appearance in local trust. Pre-trust may only name
/// peers that appear in local trust.
#[derive(Debug, Clone, PartialEq)]
pub struct Network {
    peers: Vec<String>,
    local_trust: Vec<TrustEdge>,
    pre_trust: Vec<PreTrust>,
}

impl Network {
    /// Reads local trust and pre-trust CSV text.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] with the input and line of the first bad record.
    pub fn from_csv(local_trust: &str, pre_trust: &str) -> Result<Network, Error> {
        let mut peers: Vec<String> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut edges = Vec::new();
        {
            let mut peer_index = |name: &str| {
                if let Some(&i) = index.get(name) {
                    return i;
                }
                let i = peers.len();
                index.insert(name.to_string(), i);
                peers.push(name.to_string());
                i
            };
            for_each_record(local_trust, Input::LocalTrust, 2, |line, fields| {
                let error = |kind| Error::new(Input::LocalTrust, line, kind);
                let [from, to, rest @ ..] = fields else {
                    return Err(error(ErrorKind::TooFewFields));
                };
                let weight = parse_weight(rest.first().copied()).map_err(error)?;
                let from = peer_index(from);
                let to = peer_index(to);
                edges.push(TrustEdge::new(from, to, weight));
                Ok(())
            })?;
        }

        let mut seeds = Vec::new();
        for_each_record(pre_trust, Input::PreTrust, 1, |line, fields| {
            let error = |kind| Error::new(Input::PreTrust, line, kind);
            let peer = *index
                .get(fields[0])
                .ok_or_else(|| error(ErrorKind::UnknownPeer(fields[0].to_string())))?;
            let weight = parse_weight(fields.get(1).copied()).map_err(error)?;
            seeds.push(PreTrust::new(peer, weight));
            Ok(())
        })?;

        Ok(Network {
            peers,
            local_trust: edges,
            pre_trust: seeds,
        })
    }

    /// Peer names, indexed by peer.
    pub fn peers(&self) -> &[String] {
        &self.peers
    }

    /// Local trust edges, in file order.
    pub fn local_trust(&self) -> &[TrustEdge] {
        &self.local_trust
    }

    /// Pre-trust entries, in file order.
    pub fn pre_trust(&self) -> &[PreTrust] {
        &self.pre_trust
    }

    /// Computes EigenTrust scores for this network.
    ///
    /// # Errors
    ///
    /// See [`eigentrust_with_options`].
    pub fn eigentrust(&self, options: &EigenTrustOptions) -> Result<TrustScores, EigenTrustError> {
        eigentrust_with_options(
            self.local_trust.iter().copied(),
            self.pre_trust.iter().copied(),
            options,
        )
    }

    /// `(name, score)` pairs, highest score first. Equal scores keep peer order.
    pub fn ranking<'a>(&'a self, scores: &TrustScores) -> Vec<(&'a str, f64)> {
        scores
            .ranking()
            .into_iter()
            .filter_map(|(i, s)| self.peers.get(i).map(|name| (name.as_str(), s)))
            .collect()
    }
}

/// A CSV record that could not be read.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    input: Input,
    line: u64,
    kind: ErrorKind,
}

impl Error {
    fn new(input: Input, line: u64, kind: ErrorKind) -> Self {
        Error { input, line, kind }
    }

    /// Which CSV text the record is in.
    pub fn input(&self) -> Input {
        self.input
    }

    /// 1-based line number of the record.
    pub fn line(&self) -> u64 {
        self.line
    }

    /// What is wrong with the record.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

/// What is wrong with a CSV record.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Not valid CSV, e.g. an unterminated quote or invalid UTF-8 inside quotes.
    Malformed(String),
    /// A local trust record without both peers.
    TooFewFields,
    /// The weight is not a number.
    InvalidWeight(String),
    /// The weight is NaN or infinite.
    NonFiniteWeight(String),
    /// The weight is negative. Distrust is not supported.
    NegativeWeight(String),
    /// A pre-trust peer that does not appear in local trust.
    UnknownPeer(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} CSV, line {}: ", self.input, self.line)?;
        match &self.kind {
            ErrorKind::Malformed(msg) => write!(f, "malformed CSV: {}", msg),
            ErrorKind::TooFewFields => f.write_str("too few fields"),
            ErrorKind::InvalidWeight(w) => write!(f, "weight {:?} is not a number", w),
            ErrorKind::NonFiniteWeight(w) => write!(f, "weight {:?} must be finite", w),
            ErrorKind::NegativeWeight(w) => {
                write!(f, "weight {:?} is negative; distrust is not supported", w)
            }
            ErrorKind::UnknownPeer(p) => write!(f, "peer {:?} does not appear in local trust", p),
        }
    }
}

impl std::error::Error for Error {}

const HEADER_NAMES: &[&str] = &[
    "i", "j", "v", "from", "to", "value", "weight", "trust", "level", "peer", "id", "score",
    "source", "target", "src", "dst", "truster", "trustee",
];

// Surrounding whitespace and quotes the CSV reader keeps, e.g. in ` "alice" `.
fn clean_field(field: &str) -> &str {
    let field = field.trim();
    field
        .strip_prefix('"')
        .and_then(|f| f.strip_suffix('"'))
        .unwrap_or(field)
        .trim()
}

// `value_column` is 2 for local trust and 1 for pre-trust.
fn is_header(fields: &[&str], value_column: usize) -> bool {
    match fields.get(value_column) {
        Some(value) => value.parse::<f64>().is_err(),
        None => fields
            .iter()
            .all(|f| HEADER_NAMES.contains(&f.to_ascii_lowercase().as_str())),
    }
}

// Calls `f(line, fields)` for every non-empty record after the optional header.
fn for_each_record<F>(data: &str, input: Input, value_column: usize, mut f: F) -> Result<(), Error>
where
    F: FnMut(u64, &[&str]) -> Result<(), Error>,
{
    let data = data.trim_start_matches('\u{feff}');
    let mut reader = ::csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());

    // The csv crate reports a record's position before any blank lines it skipped and
    // does not count them, so line numbers come from byte offsets.
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

    let mut record = ::csv::ByteRecord::new();
    let mut first = true;
    loop {
        let more = match reader.read_byte_record(&mut record) {
            Ok(more) => more,
            Err(e) => {
                let line = e.position().map_or(0, |p| line_at(p.byte()));
                return Err(Error::new(input, line, ErrorKind::Malformed(e.to_string())));
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
            let field = std::str::from_utf8(field)
                .map_err(|e| Error::new(input, line, ErrorKind::Malformed(e.to_string())))?;
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

// A weight: finite and non-negative, 1 when omitted.
fn parse_weight(field: Option<&str>) -> Result<f64, ErrorKind> {
    let Some(field) = field else {
        return Ok(1.0);
    };
    let weight = field
        .parse::<f64>()
        .map_err(|_| ErrorKind::InvalidWeight(field.to_string()))?;
    if !weight.is_finite() {
        return Err(ErrorKind::NonFiniteWeight(field.to_string()));
    }
    if weight < 0.0 {
        return Err(ErrorKind::NegativeWeight(field.to_string()));
    }
    Ok(weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(data: &str, value_column: usize) -> Result<Vec<(u64, Vec<String>)>, Error> {
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
    fn headers() {
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
    fn quoting() {
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
    fn line_numbers_survive_blank_lines() {
        let lines: Vec<u64> = records("i,j,v\na,b,1\n\nc,d,2\r\n\r\n\r\ne,f", 2)
            .unwrap()
            .into_iter()
            .map(|(l, _)| l)
            .collect();
        assert_eq!(lines, [2, 4, 7]);
    }

    #[test]
    fn weights() {
        assert_eq!(parse_weight(None), Ok(1.0));
        assert_eq!(parse_weight(Some("2.5")), Ok(2.5));
        assert!(matches!(
            parse_weight(Some("x")),
            Err(ErrorKind::InvalidWeight(_))
        ));
        for bad in ["NaN", "inf", "-inf"] {
            assert!(matches!(
                parse_weight(Some(bad)),
                Err(ErrorKind::NonFiniteWeight(_))
            ));
        }
        assert!(matches!(
            parse_weight(Some("-1")),
            Err(ErrorKind::NegativeWeight(_))
        ));
    }

    #[test]
    fn clean_fields() {
        assert_eq!(clean_field("  alice "), "alice");
        assert_eq!(clean_field("\"alice\""), "alice");
        assert_eq!(clean_field(" \" 0.5 \" "), "0.5");
        assert_eq!(clean_field("\""), "\"");
    }
}
