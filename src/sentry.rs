//! Sentry policy engine for entropy-krypton-core.
//!
//! Turns SentrySignals into concrete SentryDecision values (Keep / Throttle / Kill)
//! based on configurable thresholds.

use serde::{Deserialize, Serialize};

use crate::signals::SentrySignals;

/// Decision returned by the Sentry engine for a given job.
///
/// This is what you'd feed into your scheduler or controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentryDecision {
    Keep,
    Throttle,
    Kill,
}

/// Configuration thresholds for the Sentry engine.
///
/// Controls when a job is considered safe, needs throttling,
/// or should be killed outright.
///
/// The basic model:
/// - Compute a combined "stress" score from the signals.
/// - If it's below `max_entropy_score * soft_excess_factor`, we Keep.
/// - If it's between `max_entropy_score * soft_excess_factor` and
///   `max_entropy_score * hard_excess_factor`, we Throttle.
/// - If it's above `max_entropy_score * hard_excess_factor`, we Kill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Baseline "safe" maximum score.
    pub max_entropy_score: f64,
    /// Multiplier for the soft region (Throttle).
    pub soft_excess_factor: f64,
    /// Multiplier for the hard region (Kill).
    pub hard_excess_factor: f64,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            max_entropy_score: 1.0,
            soft_excess_factor: 1.1,
            hard_excess_factor: 1.5,
        }
    }
}

impl SentryConfig {
    /// Load SentryConfig from a JSON string.
    ///
    /// Example JSON:
    /// ```json
    /// {
    ///   "max_entropy_score": 1.0,
    ///   "soft_excess_factor": 1.2,
    ///   "hard_excess_factor": 1.5
    /// }
    /// ```
    pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// Load SentryConfig from a JSON file path.
    pub fn from_json_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let cfg = serde_json::from_str(&text)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        Ok(cfg)
    }
}

/// Policy engine that turns SentrySignals into SentryDecision values.
///
/// In a real deployment this would sit between telemetry and
/// your job scheduler / controller.
#[derive(Debug, Clone)]
pub struct SentryEngine {
    config: SentryConfig,
}

impl Default for SentryEngine {
    fn default() -> Self {
        Self::new(SentryConfig::default())
    }
}

impl SentryEngine {
    /// Create a new SentryEngine with the given configuration.
    pub fn new(config: SentryConfig) -> Self {
        Self { config }
    }

    /// Convenience constructor using the default config.
    pub fn with_default_config() -> Self {
        Self::new(SentryConfig::default())
    }

    /// Decide whether to keep, throttle, or kill a job based on its signals.
    ///
    /// `job_id` is currently unused, but kept so this can be extended later
    /// (per-job history, logging, etc.).
    pub fn decide(&self, _job_id: &str, signals: &SentrySignals) -> SentryDecision {
        // Combine the three signal dimensions into a single "stress" score.
        // You can tune this later; for now we use a simple sum.
        let stress_score = signals.entropy_score + signals.jitter_score + signals.load_score;

        let soft_threshold = self.config.max_entropy_score * self.config.soft_excess_factor;
        let hard_threshold = self.config.max_entropy_score * self.config.hard_excess_factor;

        if stress_score >= hard_threshold {
            SentryDecision::Kill
        } else if stress_score >= soft_threshold {
            SentryDecision::Throttle
        } else {
            SentryDecision::Keep
        }
    }
}
