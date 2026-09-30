//! [EigenTrust](https://nlp.stanford.edu/pubs/eigentrust.pdf) reputation scores for
//! peer-to-peer networks, social graphs and other webs of trust.
//!
//! Each peer says how much it trusts some other peers (*local trust*). A few peers are
//! trusted from the start (*pre-trust*, or seeds). EigenTrust combines both into one global
//! score per peer: trust flows from the seeds along the network, so a cluster of fake
//! accounts that only vouch for each other ends up with almost nothing. Without seeds it
//! behaves like PageRank.
//!
//! This crate implements the algorithm from Kamvar, Schlosser and Garcia-Molina (2003) as
//! a power iteration over a sparse matrix. It runs on any Rust target, including
//! `wasm32-unknown-unknown`.
//!
//! # Example
//!
//! ```
//! use eigentrust::{eigentrust, PreTrust, TrustEdge};
//!
//! // peer 0 is the seed; 0 trusts 1 twice as much as 2; 1 and 2 trust each other
//! let local_trust = [
//!     TrustEdge::new(0, 1, 2.0),
//!     TrustEdge::new(0, 2, 1.0),
//!     TrustEdge::new(1, 2, 1.0),
//!     TrustEdge::new(2, 1, 1.0),
//! ];
//! let pre_trust = [PreTrust::new(0, 1.0)];
//!
//! let result = eigentrust(local_trust, pre_trust)?;
//! let scores = result.scores();
//! assert_eq!(scores.len(), 3);
//! assert!(scores[1] > scores[2]);
//! assert!((scores.iter().sum::<f64>() - 1.0).abs() < 1e-9);
//! # Ok::<(), eigentrust::EigenTrustError>(())
//! ```
//!
//! Use [`eigentrust_with_options`] to change `alpha`, the convergence threshold or the
//! iteration limit (see [`EigenTrustOptions`]).
//!
//! # Input
//!
//! - **Peers** are `usize` indices. The network has `n = max index + 1` peers, taken over
//!   both inputs; every index below that is a peer, even one that appears nowhere.
//!   Keep indices dense.
//! - **Weights** must be finite and non-negative. Zero means no trust. Negative trust
//!   (distrust) is rejected with [`EigenTrustError::NegativeWeight`].
//! - **Local trust** is normalized per truster: each peer's outgoing weights are scaled to
//!   sum to 1, so only their ratios matter. A peer that trusts nobody passes its share on
//!   according to pre-trust.
//! - **Pre-trust** is normalized to sum to 1. If it is empty or all zero, every peer is
//!   equally pre-trusted.
//! - **Duplicates**: a repeated `(from, to)` edge or a repeated pre-trusted peer keeps the
//!   last weight given.
//! - Self-trust (`from == to`) is allowed and treated like any other edge.
//!
//! # Output
//!
//! [`TrustScores`] holds one score per peer, indexed by peer. Scores are non-negative and sum
//! to 1. It also reports the number of iterations and the final residual.
//!
//! # Convergence
//!
//! Each iteration computes `t = (1 - alpha) * Cᵀ t + alpha * p`, starting from `t = p`,
//! where `C` is the normalized local trust and `p` the normalized pre-trust. It stops when
//! the L2 norm of the change is at most `epsilon` (default `1e-6 / n`). With `alpha > 0` the
//! error shrinks by a factor of `1 - alpha` per iteration. With `alpha = 0` some networks
//! oscillate forever; after `max_iterations` (default 10,000) the result is
//! [`EigenTrustError::NotConverged`].
//!
//! Sums are compensated and always taken in the same order, so results are reproducible
//! bit for bit, with or without the `parallel` feature.
//!
//! # Features
//!
//! - `csv`: [`csv::Network`] reads named peers from CSV text.
//! - `parallel`: multithreaded iteration with rayon, for networks with many thousands of
//!   peers.
//!
//! # Other interfaces
//!
//! The same implementation powers a command-line tool (the `eigentrust-cli` crate) and
//! WebAssembly bindings for the browser (the `eigentrust-wasm` crate in the repository).
//! Both are thin adapters over this crate's public API.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod algorithm;
mod error;
mod options;
mod scores;
mod types;

#[cfg(feature = "csv")]
pub mod csv;

// the README's Rust examples run as doc tests
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

pub use error::{EigenTrustError, Input};
pub use options::EigenTrustOptions;
pub use scores::TrustScores;
pub use types::{PreTrust, TrustEdge};

/// Computes EigenTrust scores with the default [`EigenTrustOptions`].
///
/// See the [crate documentation](crate) for how the inputs are interpreted.
///
/// # Errors
///
/// Returns an [`EigenTrustError`] for invalid weights, an empty network, or if the scores
/// do not converge.
pub fn eigentrust(
    local_trust: impl IntoIterator<Item = TrustEdge>,
    pre_trust: impl IntoIterator<Item = PreTrust>,
) -> Result<TrustScores, EigenTrustError> {
    algorithm::run(local_trust, pre_trust, &EigenTrustOptions::default())
}

/// Computes EigenTrust scores with custom options.
///
/// ```
/// use eigentrust::{eigentrust_with_options, EigenTrustOptions, PreTrust, TrustEdge};
///
/// let options = EigenTrustOptions::default().with_alpha(0.15);
/// let result = eigentrust_with_options(
///     [TrustEdge::new(0, 1, 1.0), TrustEdge::new(1, 0, 1.0)],
///     [PreTrust::new(0, 1.0)],
///     &options,
/// )?;
/// assert!(result.scores()[0] > result.scores()[1]);
/// # Ok::<(), eigentrust::EigenTrustError>(())
/// ```
///
/// # Errors
///
/// Returns an [`EigenTrustError`] for invalid options or weights, an empty network, or if
/// the scores do not converge within [`EigenTrustOptions::max_iterations`].
pub fn eigentrust_with_options(
    local_trust: impl IntoIterator<Item = TrustEdge>,
    pre_trust: impl IntoIterator<Item = PreTrust>,
    options: &EigenTrustOptions,
) -> Result<TrustScores, EigenTrustError> {
    algorithm::run(local_trust, pre_trust, options)
}
