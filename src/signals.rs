/// Input signals for the Sentry engine.
///
/// In a real deployment these might be normalized to [0,1] or z-scores.
/// Here we keep it simple: higher = "worse".
#[derive(Debug, Clone)]
pub struct SentrySignals {
    /// Overall "entropy health" score (e.g. variance, unpredictability, etc.).
    pub entropy_score: f64,
    /// Instability or jitter in the signal.
    pub jitter_score: f64,
    /// A simple load indicator (e.g. CPU, queue depth, GPU utilization).
    pub load_score: f64,
}

impl SentrySignals {
    /// Construct SentrySignals directly from raw scores.
    ///
    /// This is mainly used in tests and small demos; in a real system,
    /// you'd usually go through EntropyMetrics.
    pub fn from_raw(entropy_score: f64, jitter_score: f64, load_score: f64) -> Self {
        Self {
            entropy_score,
            jitter_score,
            load_score,
        }
    }
}
