/// One peer's trust in another: `from` trusts `to` with `weight`.
///
/// Weights are relative: each truster's weights are scaled to sum to 1, so `(0, 1, 2.0)`
/// and `(0, 2, 1.0)` mean peer 0 gives two thirds of its trust to 1 and one third to 2.
///
/// ```
/// use eigentrust::TrustEdge;
///
/// let edge = TrustEdge::new(0, 1, 2.0);
/// assert_eq!(edge, (0, 1, 2.0).into());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrustEdge {
    /// The truster.
    pub from: usize,
    /// The trusted peer.
    pub to: usize,
    /// How much `from` trusts `to`. Finite and non-negative; 0 means no trust.
    pub weight: f64,
}

impl TrustEdge {
    /// Creates an edge.
    pub const fn new(from: usize, to: usize, weight: f64) -> Self {
        TrustEdge { from, to, weight }
    }
}

impl From<(usize, usize, f64)> for TrustEdge {
    fn from((from, to, weight): (usize, usize, f64)) -> Self {
        TrustEdge { from, to, weight }
    }
}

/// A seed peer: someone trusted before any local trust is considered.
///
/// Pre-trust weights are relative and scaled to sum to 1. Trust flows out from these peers,
/// and every iteration returns a share `alpha` of it to them.
///
/// ```
/// use eigentrust::PreTrust;
///
/// assert_eq!(PreTrust::new(0, 1.0), (0, 1.0).into());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreTrust {
    /// The seed peer.
    pub peer: usize,
    /// How much it is trusted. Finite and non-negative.
    pub weight: f64,
}

impl PreTrust {
    /// Creates a pre-trust entry.
    pub const fn new(peer: usize, weight: f64) -> Self {
        PreTrust { peer, weight }
    }
}

impl From<(usize, f64)> for PreTrust {
    fn from((peer, weight): (usize, f64)) -> Self {
        PreTrust { peer, weight }
    }
}
