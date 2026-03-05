//! Sentry policy engine for entropy-krypton-core.
//!
//! Turns SentrySignals into concrete SentryDecision values (Keep / Throttle / Kill)
//! based on configurable thresholds.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

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
///
/// # Collapsed Throttle band
///
/// Setting `soft_excess_factor == hard_excess_factor` collapses the Throttle
/// band to zero width.  Any stress score at or above the shared threshold
/// jumps directly to `Kill`, skipping `Throttle` entirely.  This is
/// intentional and can be used to implement a binary Keep / Kill policy.
/// Validate using [`SentryConfig::validate`] if this is undesired.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Baseline "safe" maximum score.
    pub max_entropy_score: f64,
    /// Multiplier for the soft region (Throttle).
    pub soft_excess_factor: f64,
    /// Multiplier for the hard region (Kill).
    pub hard_excess_factor: f64,
}

#[derive(Debug)]
pub enum SentryConfigError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for SentryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON parse error: {err}"),
            Self::Invalid(msg) => write!(f, "invalid sentry config: {msg}"),
        }
    }
}

impl Error for SentryConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::Invalid(_) => None,
        }
    }
}

impl From<std::io::Error> for SentryConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SentryConfigError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
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
    /// Validate config invariants and numeric sanity.
    pub fn validate(&self) -> Result<(), SentryConfigError> {
        if !self.max_entropy_score.is_finite() || self.max_entropy_score <= 0.0 {
            return Err(SentryConfigError::Invalid(
                "max_entropy_score must be finite and > 0".to_string(),
            ));
        }
        if !self.soft_excess_factor.is_finite() || self.soft_excess_factor <= 0.0 {
            return Err(SentryConfigError::Invalid(
                "soft_excess_factor must be finite and > 0".to_string(),
            ));
        }
        if !self.hard_excess_factor.is_finite() || self.hard_excess_factor <= 0.0 {
            return Err(SentryConfigError::Invalid(
                "hard_excess_factor must be finite and > 0".to_string(),
            ));
        }
        if self.hard_excess_factor < self.soft_excess_factor {
            return Err(SentryConfigError::Invalid(
                "hard_excess_factor must be >= soft_excess_factor".to_string(),
            ));
        }
        Ok(())
    }

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
    pub fn from_json_str(s: &str) -> Result<Self, SentryConfigError> {
        let cfg: Self = serde_json::from_str(s)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Load SentryConfig from a JSON file path.
    pub fn from_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SentryConfigError> {
        let text = std::fs::read_to_string(path)?;
        let cfg: Self = serde_json::from_str(&text)?;
        cfg.validate()?;
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
        if !signals.entropy_score.is_finite()
            || !signals.jitter_score.is_finite()
            || !signals.load_score.is_finite()
        {
            return SentryDecision::Kill;
        }

        // Combine the three signal dimensions into a single "stress" score.
        // You can tune this later; for now we use a simple sum.
        let stress_score = signals.entropy_score + signals.jitter_score + signals.load_score;

        let soft_threshold = self.config.max_entropy_score * self.config.soft_excess_factor;
        let hard_threshold = self.config.max_entropy_score * self.config.hard_excess_factor;
        if !soft_threshold.is_finite()
            || !hard_threshold.is_finite()
            || hard_threshold < soft_threshold
        {
            return SentryDecision::Kill;
        }

        if stress_score >= hard_threshold {
            SentryDecision::Kill
        } else if stress_score >= soft_threshold {
            SentryDecision::Throttle
        } else {
            SentryDecision::Keep
        }
    }
}
