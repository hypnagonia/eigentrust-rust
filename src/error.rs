use std::fmt;

/// Which input an [`EigenTrustError`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    /// The local trust edges.
    LocalTrust,
    /// The pre-trust (seed) entries.
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

/// Why [`eigentrust`](crate::eigentrust) could not compute scores.
///
/// Input positions are 0-based indices into the iterator that was passed in.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EigenTrustError {
    /// `alpha` is not a finite number in `[0, 1]`.
    InvalidAlpha(f64),
    /// `epsilon` is not a finite number greater than zero.
    InvalidEpsilon(f64),
    /// `max_iterations` is zero.
    InvalidMaxIterations,
    /// A weight is NaN or infinite.
    NonFiniteWeight {
        /// The input containing the weight.
        input: Input,
        /// Position of the item in that input.
        position: usize,
    },
    /// A weight is negative. Distrust is not supported.
    NegativeWeight {
        /// The input containing the weight.
        input: Input,
        /// Position of the item in that input.
        position: usize,
        /// The rejected weight.
        weight: f64,
    },
    /// A peer index of `usize::MAX`, which leaves no room to count the peers.
    InvalidPeer {
        /// The input containing the index.
        input: Input,
        /// Position of the item in that input.
        position: usize,
    },
    /// The weights of one truster, or all of pre-trust, add up to more than `f64::MAX`.
    WeightOverflow(Input),
    /// Neither input mentions any peer.
    EmptyNetwork,
    /// The scores did not settle within `max_iterations`. With `alpha` near zero some
    /// networks oscillate instead of converging.
    NotConverged {
        /// Iterations performed.
        iterations: usize,
        /// Change between the last two iterations (L2 norm).
        residual: f64,
    },
}

impl fmt::Display for EigenTrustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EigenTrustError::InvalidAlpha(a) => write!(f, "alpha must be in [0, 1], got {}", a),
            EigenTrustError::InvalidEpsilon(e) => {
                write!(f, "epsilon must be finite and greater than 0, got {}", e)
            }
            EigenTrustError::InvalidMaxIterations => {
                f.write_str("max_iterations must be at least 1")
            }
            EigenTrustError::NonFiniteWeight { input, position } => {
                write!(f, "{} item {}: weight must be finite", input, position)
            }
            EigenTrustError::NegativeWeight {
                input,
                position,
                weight,
            } => write!(
                f,
                "{} item {}: weight {} is negative; distrust is not supported",
                input, position, weight
            ),
            EigenTrustError::InvalidPeer { input, position } => {
                write!(
                    f,
                    "{} item {}: peer index usize::MAX is not allowed",
                    input, position
                )
            }
            EigenTrustError::WeightOverflow(input) => {
                write!(
                    f,
                    "{} weights are too large: their sum overflows f64",
                    input
                )
            }
            EigenTrustError::EmptyNetwork => f.write_str("the network has no peers"),
            EigenTrustError::NotConverged {
                iterations,
                residual,
            } => write!(
                f,
                "did not converge in {} iterations (residual {:.3e}); \
                 a small alpha on a periodic trust graph can oscillate, try a larger alpha",
                iterations, residual
            ),
        }
    }
}

impl std::error::Error for EigenTrustError {}
