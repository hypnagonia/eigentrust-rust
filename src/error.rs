use std::fmt;

/// Which CSV input an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    LocalTrust,
    PreTrust,
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Input::LocalTrust => "local trust",
            Input::PreTrust => "pre-trust",
        })
    }
}

/// What is wrong with a single CSV record.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordError {
    /// The CSV itself is malformed, e.g. an unterminated quote.
    Malformed(String),
    TooFewFields,
    InvalidWeight(String),
    NonFiniteWeight(String),
    /// Distrust has no defined effect on the scores yet, so it is rejected.
    NegativeWeight(String),
    /// A pre-trust peer that never appears in local trust.
    UnknownPeer(String),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::Malformed(msg) => write!(f, "malformed CSV: {}", msg),
            RecordError::TooFewFields => f.write_str("too few fields"),
            RecordError::InvalidWeight(w) => write!(f, "weight {:?} is not a number", w),
            RecordError::NonFiniteWeight(w) => write!(f, "weight {:?} must be finite", w),
            RecordError::NegativeWeight(w) => {
                write!(f, "weight {:?} is negative; distrust is not supported", w)
            }
            RecordError::UnknownPeer(p) => {
                write!(f, "peer {:?} does not appear in local trust", p)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A CSV record could not be used. `line` is 1-based.
    Record {
        input: Input,
        line: u64,
        error: RecordError,
    },
    EmptyLocalTrust,
    InvalidAlpha(f64),
    DimensionMismatch,
    /// The iteration produced NaN or infinity.
    NonFiniteScores,
    NotConverged {
        iterations: usize,
        alpha: f64,
    },
    InvalidUtf8(Input),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Record { input, line, error } => {
                write!(f, "{} CSV, line {}: {}", input, line, error)
            }
            Error::EmptyLocalTrust => f.write_str("local trust is empty"),
            Error::InvalidAlpha(a) => write!(f, "alpha must be in [0, 1], got {}", a),
            Error::DimensionMismatch => f.write_str("dimension mismatch"),
            Error::NonFiniteScores => f.write_str("trust scores are not finite"),
            Error::NotConverged { iterations, alpha } => write!(
                f,
                "did not converge in {} iterations with alpha {}; \
                 a small alpha on a periodic trust graph can oscillate, try a larger alpha",
                iterations, alpha
            ),
            Error::InvalidUtf8(input) => write!(f, "{} CSV is not valid UTF-8", input),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
