use crate::signals::SentrySignals;

/// Decision the Sentry makes for a given job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentryDecision {
    Keep,
    Throttle,
    Kill,
}

/// Static configuration for the Sentry engine.
///
/// For v0.1 this is simple threshold logic.
/// In a more advanced version you'd include weights, hysteresis, etc.
#[derive(Debug, Clone)]
pub struct SentryConfig {
    pub max_entropy_score: f64,
    pub max_jitter_score: f64,
    pub max_load_score: f64,
    /// If any signal exceeds this multiple of its limit, go straight to Kill.
    pub hard_excess_factor: f64,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            max_entropy_score: 1.0,
            max_jitter_score: 0.5,
            max_load_score: 0.85,
            hard_excess_factor: 2.0,
        }
    }
}

/// The SentryEngine evaluates signals and returns a decision for each job.
#[derive(Debug, Clone)]
pub struct SentryEngine {
    config: SentryConfig,
}

impl SentryEngine {
    pub fn new(config: SentryConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SentryConfig::default())
    }

    /// Decide whether to keep, throttle, or kill a job based on its signals.
    pub fn decide(&self, _job_id: &str, signals: &SentrySignals) -> SentryDecision {
        let c = &self.config;

        let entropy_ratio = safe_ratio(signals.entropy_score, c.max_entropy_score);
        let jitter_ratio = safe_ratio(signals.jitter_score, c.max_jitter_score);
        let load_ratio = safe_ratio(signals.load_score, c.max_load_score);

        let max_ratio = entropy_ratio.max(jitter_ratio).max(load_ratio);

        if max_ratio >= c.hard_excess_factor {
            SentryDecision::Kill
        } else if max_ratio > 1.0 {
            SentryDecision::Throttle
        } else {
            SentryDecision::Keep
        }
    }
}

fn safe_ratio(value: f64, limit: f64) -> f64 {
    if limit <= 0.0 {
        return 0.0;
    }
    value / limit
}
