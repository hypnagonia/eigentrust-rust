use crate::EigenTrustError;

/// Settings for [`eigentrust_with_options`](crate::eigentrust_with_options).
///
/// Start from [`EigenTrustOptions::default()`] and change what you need. Values are checked
/// when the computation starts, before any work is done.
///
/// ```
/// use eigentrust::EigenTrustOptions;
///
/// let options = EigenTrustOptions::default()
///     .with_alpha(0.2)
///     .with_max_iterations(500);
/// assert_eq!(options.alpha(), 0.2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EigenTrustOptions {
    alpha: f64,
    epsilon: Option<f64>,
    max_iterations: usize,
}

impl EigenTrustOptions {
    /// Default [`alpha`](Self::alpha).
    pub const DEFAULT_ALPHA: f64 = 0.5;
    /// Default [`max_iterations`](Self::max_iterations).
    pub const DEFAULT_MAX_ITERATIONS: usize = 10_000;

    /// Same as [`EigenTrustOptions::default()`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets how strongly each iteration pulls trust back to the pre-trusted peers, in
    /// `[0, 1]`.
    ///
    /// Higher values keep trust close to the seeds, which starves Sybil clusters; lower
    /// values let it travel further. At `0` pre-trust only sets the starting point, and some
    /// networks then oscillate instead of converging.
    #[must_use]
    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha;
        self
    }

    /// Sets the convergence threshold: iteration stops once the L2 norm of the change in
    /// scores between two iterations is at most `epsilon`. Must be finite and positive.
    ///
    /// By default it is `1e-6 / n` for a network of `n` peers.
    #[must_use]
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = Some(epsilon);
        self
    }

    /// Sets the iteration limit. Reaching it returns
    /// [`EigenTrustError::NotConverged`]. Must be at least 1.
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Share of trust returned to the pre-trusted peers each iteration.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// The fixed convergence threshold, or `None` for the default `1e-6 / n`.
    pub fn epsilon(&self) -> Option<f64> {
        self.epsilon
    }

    /// The iteration limit.
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    pub(crate) fn validate(&self) -> Result<(), EigenTrustError> {
        if !(0.0..=1.0).contains(&self.alpha) {
            return Err(EigenTrustError::InvalidAlpha(self.alpha));
        }
        if let Some(e) = self.epsilon {
            if !(e.is_finite() && e > 0.0) {
                return Err(EigenTrustError::InvalidEpsilon(e));
            }
        }
        if self.max_iterations == 0 {
            return Err(EigenTrustError::InvalidMaxIterations);
        }
        Ok(())
    }

    pub(crate) fn epsilon_for(&self, peers: usize) -> f64 {
        self.epsilon.unwrap_or(1e-6 / peers as f64)
    }
}

impl Default for EigenTrustOptions {
    /// `alpha` 0.5, `epsilon` `1e-6 / n`, `max_iterations` 10,000.
    fn default() -> Self {
        EigenTrustOptions {
            alpha: Self::DEFAULT_ALPHA,
            epsilon: None,
            max_iterations: Self::DEFAULT_MAX_ITERATIONS,
        }
    }
}
