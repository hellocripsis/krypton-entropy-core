use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::VecDeque;
use std::fmt;

use crate::metrics::EntropyMetrics;
use crate::sentry::SentryDecision;

/// Error returned when an [`EntropyConfig`] fails validation.
#[derive(Debug)]
pub struct EntropyConfigError(pub String);

impl fmt::Display for EntropyConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid entropy config: {}", self.0)
    }
}

impl std::error::Error for EntropyConfigError {}

/// Configuration for the entropy engine + sentry decision.
#[derive(Debug, Clone, Copy)]
pub struct EntropyConfig {
    /// Minimum samples before we trust the metrics at all.
    pub min_samples: u64,
    /// Expected center for bit-density (p), must be in (0.0, 1.0).
    pub mean_center: f64,
    /// Allowed deviation around `mean_center` before we worry. Must be > 0.
    pub mean_tolerance: f64,
    /// Jitter (stddev) level where we start throttling. Must be > 0 and <= `kill_jitter`.
    pub max_jitter: f64,
    /// Jitter (stddev) level where we kill. Must be >= `max_jitter`.
    pub kill_jitter: f64,
    /// Number of recent samples kept for rolling metrics. Must be >= 1.
    pub window_size: usize,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            min_samples: 256,
            mean_center: 0.5,
            mean_tolerance: 0.05,
            // Thresholds calibrated to stddev of bit-density for healthy OsRng
            // (expected stddev ≈ 0.0625). max_jitter ≈ 2×, kill_jitter ≈ 3.5×.
            max_jitter: 0.12,
            kill_jitter: 0.22,
            window_size: 16_384,
        }
    }
}

impl EntropyConfig {
    /// More sensitive profile – easier to trigger Throttle/Kill.
    pub fn aggressive() -> Self {
        Self {
            min_samples: 128,
            mean_center: 0.5,
            mean_tolerance: 0.03,
            max_jitter: 0.09,
            kill_jitter: 0.17,
            window_size: 8_192,
        }
    }

    /// Validate config invariants and numeric sanity.
    ///
    /// Called automatically by [`EntropyEngine::with_config`] and
    /// [`EntropyEngine::try_with_config`]. Exposed publicly so callers that
    /// build configs dynamically can surface errors early.
    pub fn validate(&self) -> Result<(), EntropyConfigError> {
        if self.window_size == 0 {
            return Err(EntropyConfigError("window_size must be >= 1".into()));
        }
        if !self.mean_center.is_finite() || self.mean_center <= 0.0 || self.mean_center >= 1.0 {
            return Err(EntropyConfigError(
                "mean_center must be finite and in (0.0, 1.0)".into(),
            ));
        }
        if !self.mean_tolerance.is_finite() || self.mean_tolerance <= 0.0 {
            return Err(EntropyConfigError(
                "mean_tolerance must be finite and > 0".into(),
            ));
        }
        if !self.max_jitter.is_finite() || self.max_jitter <= 0.0 {
            return Err(EntropyConfigError(
                "max_jitter must be finite and > 0".into(),
            ));
        }
        if !self.kill_jitter.is_finite() || self.kill_jitter <= 0.0 {
            return Err(EntropyConfigError(
                "kill_jitter must be finite and > 0".into(),
            ));
        }
        if self.kill_jitter < self.max_jitter {
            return Err(EntropyConfigError(
                "kill_jitter must be >= max_jitter".into(),
            ));
        }
        Ok(())
    }
}

/// Entropy engine: samples OsRng, tracks metrics, and produces a decision.
///
/// Metrics are maintained in **O(1)** per sample via incremental sum /
/// sum-of-squares tracking over a bounded rolling window.  `jitter` in the
/// returned [`EntropyMetrics`] is the **standard deviation** of the window.
#[derive(Debug)]
pub struct EntropyEngine {
    cfg: EntropyConfig,
    rng: OsRng,
    /// Rolling window — retained so evicted values can be subtracted from sums.
    window: VecDeque<f64>,
    /// Monotonically increasing total; drives the `min_samples` warm-up gate.
    total_samples: u64,
    /// Running sum of values in `window`.
    sum_x: f64,
    /// Running sum of squared values in `window`.
    sum_x2: f64,
}

impl Default for EntropyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyEngine {
    /// Create an engine with the given config.
    ///
    /// # Panics
    ///
    /// Panics if `cfg` fails [`EntropyConfig::validate`].  Use
    /// [`EntropyEngine::try_with_config`] to handle invalid configs without
    /// panicking.
    pub fn with_config(cfg: EntropyConfig) -> Self {
        cfg.validate().unwrap_or_else(|e| panic!("{e}"));
        Self::build(cfg)
    }

    /// Fallible constructor — returns an error instead of panicking if `cfg`
    /// fails validation.
    pub fn try_with_config(cfg: EntropyConfig) -> Result<Self, EntropyConfigError> {
        cfg.validate()?;
        Ok(Self::build(cfg))
    }

    /// Create engine with default config.
    pub fn new() -> Self {
        Self::with_config(EntropyConfig::default())
    }

    fn build(cfg: EntropyConfig) -> Self {
        Self {
            cfg,
            rng: OsRng,
            window: VecDeque::new(),
            total_samples: 0,
            sum_x: 0.0,
            sum_x2: 0.0,
        }
    }

    /// Reset all sample state, clearing the rolling window and running sums.
    ///
    /// The configuration is preserved. `total_samples` is also reset, so the
    /// warm-up gate re-applies until `min_samples` is reached again.
    pub fn reset(&mut self) {
        self.window.clear();
        self.total_samples = 0;
        self.sum_x = 0.0;
        self.sum_x2 = 0.0;
    }

    /// Take one sample from OsRng; returns `p` (bit-density in `[0, 1]`).
    pub fn sample(&mut self) -> f64 {
        let x: u64 = self.rng.next_u64();
        let bits_set = x.count_ones() as f64;
        let p = bits_set / 64.0;

        self.window.push_back(p);
        self.sum_x += p;
        self.sum_x2 += p * p;
        self.total_samples = self.total_samples.saturating_add(1);

        if self.window.len() > self.cfg.window_size {
            if let Some(old) = self.window.pop_front() {
                self.sum_x -= old;
                self.sum_x2 -= old * old;
            }
        }

        p
    }

    /// Compute current metrics over the rolling sample window.
    ///
    /// This is **O(1)** — derived from running sums maintained during
    /// [`sample`].  `jitter` is the standard deviation of the current window.
    pub fn metrics(&self) -> EntropyMetrics {
        let n = self.window.len();
        if n == 0 {
            return EntropyMetrics {
                mean: 0.0,
                variance: 0.0,
                jitter: 0.0,
                sample_count: 0,
            };
        }

        let n_f = n as f64;
        let mean = self.sum_x / n_f;
        // Clamp to zero to absorb floating-point rounding errors near zero.
        let variance = ((self.sum_x2 / n_f) - mean * mean).max(0.0);
        let jitter = variance.sqrt();

        EntropyMetrics {
            mean,
            variance,
            jitter,
            sample_count: n,
        }
    }

    /// Compute a sentry decision from current metrics.
    pub fn decision(&self) -> SentryDecision {
        // Not enough data yet — do nothing dramatic.
        if self.total_samples < self.cfg.min_samples {
            return SentryDecision::Keep;
        }

        let m = self.metrics();
        let mean_delta = (m.mean - self.cfg.mean_center).abs();

        // Hard fail conditions.
        if mean_delta > self.cfg.mean_tolerance || m.jitter > self.cfg.kill_jitter {
            return SentryDecision::Kill;
        }

        // Softer anomaly conditions.
        if mean_delta > self.cfg.mean_tolerance * 0.5 || m.jitter > self.cfg.max_jitter {
            return SentryDecision::Throttle;
        }

        SentryDecision::Keep
    }
}
