use crate::metrics::EntropyMetrics;

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
    /// Construct signals from metrics and a raw load value.
    ///
    /// Scale is deliberately simple:
    /// - entropy_score ~ variance
    /// - jitter_score  ~ jitter
    /// - load_score    ~ raw load (assumed 0..1-ish in the demo)
    pub fn from_metrics(metrics: &EntropyMetrics, load: f64) -> Self {
        Self {
            entropy_score: metrics.variance,
            jitter_score: metrics.jitter,
            load_score: load,
        }
    }
}
