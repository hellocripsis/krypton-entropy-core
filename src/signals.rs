/// Input signals for the Sentry engine.
///
/// All three scores should be normalised so that values near **0.0 represent
/// healthy / baseline conditions** and higher values represent increasing
/// stress or anomaly severity.
///
/// [`SentryEngine`] sums all three scores and compares the total against
/// config thresholds.  There is no enforced upper bound, but keeping each
/// dimension in roughly `[0.0, 1.0]` ensures the default [`SentryConfig`]
/// thresholds are meaningful.  **Raw unbounded values** (e.g. CPU
/// percentages, queue depths, latency in milliseconds) must be normalised
/// before being passed in — feeding them directly will cause spurious `Kill`
/// decisions against default thresholds.
///
/// [`SentryEngine`]: crate::sentry::SentryEngine
/// [`SentryConfig`]: crate::sentry::SentryConfig
#[derive(Debug, Clone)]
pub struct SentrySignals {
    /// Overall "entropy health" score (e.g. variance, unpredictability).
    pub entropy_score: f64,
    /// Instability or jitter in the signal.
    pub jitter_score: f64,
    /// A simple load indicator (e.g. CPU, queue depth, GPU utilisation).
    pub load_score: f64,
}

impl SentrySignals {
    /// Construct `SentrySignals` directly from raw scores.
    ///
    /// This is mainly used in tests and small demos; in a real system,
    /// you'd usually go through [`EntropyMetrics`].
    ///
    /// [`EntropyMetrics`]: crate::metrics::EntropyMetrics
    pub fn from_raw(entropy_score: f64, jitter_score: f64, load_score: f64) -> Self {
        Self {
            entropy_score,
            jitter_score,
            load_score,
        }
    }
}
