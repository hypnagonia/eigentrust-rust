/// Global trust scores, one per peer, plus how the computation went.
///
/// Scores are indexed by peer: `scores()[i]` belongs to peer `i`. They are non-negative and
/// add up to 1 (up to floating point rounding).
#[derive(Debug, Clone, PartialEq)]
pub struct TrustScores {
    scores: Vec<f64>,
    iterations: usize,
    residual: f64,
}

impl TrustScores {
    pub(crate) fn new(scores: Vec<f64>, iterations: usize, residual: f64) -> Self {
        TrustScores {
            scores,
            iterations,
            residual,
        }
    }

    /// The score of every peer, indexed by peer.
    pub fn scores(&self) -> &[f64] {
        &self.scores
    }

    /// The score of one peer, or `None` if the index is outside the network.
    pub fn get(&self, peer: usize) -> Option<f64> {
        self.scores.get(peer).copied()
    }

    /// Number of peers.
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// Always `false`: a computed network has at least one peer.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }

    /// `(peer, score)` pairs, highest score first. Equal scores keep peer order.
    pub fn ranking(&self) -> Vec<(usize, f64)> {
        let mut ranking: Vec<(usize, f64)> = self.scores.iter().copied().enumerate().collect();
        ranking.sort_by(|a, b| b.1.total_cmp(&a.1));
        ranking
    }

    /// Power iterations performed.
    pub fn iterations(&self) -> usize {
        self.iterations
    }

    /// Change between the last two iterations (L2 norm). At most the convergence threshold.
    pub fn residual(&self) -> f64 {
        self.residual
    }

    /// The scores, indexed by peer.
    pub fn into_scores(self) -> Vec<f64> {
        self.scores
    }
}
