use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::VecDeque;

use crate::metrics::EntropyMetrics;
use crate::sentry::SentryDecision;

/// Configuration for the entropy engine + sentry decision.
#[derive(Debug, Clone, Copy)]
pub struct EntropyConfig {
    /// Minimum samples before we trust the metrics at all.
    pub min_samples: u64,
    /// Expected center for bit-density (p), usually 0.5.
    pub mean_center: f64,
    /// Allowed deviation around mean_center before we worry.
    pub mean_tolerance: f64,
    /// Jitter level where we start throttling.
    pub max_jitter: f64,
    /// Jitter level where we kill.
    pub kill_jitter: f64,
    /// Number of recent samples kept for rolling metrics.
    pub window_size: usize,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            min_samples: 256,
            mean_center: 0.5,
            mean_tolerance: 0.05, // +/- 0.05 around 0.5
            max_jitter: 0.10,
            kill_jitter: 0.18,
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
            max_jitter: 0.07,
            kill_jitter: 0.14,
            window_size: 8_192,
        }
    }
}

/// Entropy engine: samples OsRng, tracks metrics, and produces a decision.
#[derive(Debug)]
pub struct EntropyEngine {
    cfg: EntropyConfig,
    rng: OsRng,
    samples: VecDeque<f64>,
    total_samples: u64,
}

impl Default for EntropyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyEngine {
    /// Create engine with the given config.
    pub fn with_config(cfg: EntropyConfig) -> Self {
        Self {
            cfg,
            rng: OsRng,
            samples: VecDeque::new(),
            total_samples: 0,
        }
    }

    /// Create engine with default config.
    pub fn new() -> Self {
        Self::with_config(EntropyConfig::default())
    }

    /// Take one sample from OsRng, return `p` (bit-density in [0,1]).
    pub fn sample(&mut self) -> f64 {
        let x: u64 = self.rng.next_u64();
        let bits_set = x.count_ones() as f64;
        let p = bits_set / 64.0;
        self.samples.push_back(p);
        self.total_samples = self.total_samples.saturating_add(1);
        if self.samples.len() > self.cfg.window_size {
            let _ = self.samples.pop_front();
        }
        p
    }

    /// Compute current metrics over the rolling sample window.
    pub fn metrics(&self) -> EntropyMetrics {
        let sample_count = self.samples.len();
        if sample_count == 0 {
            return EntropyMetrics {
                mean: 0.0,
                variance: 0.0,
                jitter: 0.0,
                sample_count: 0,
            };
        }

        let mean = self.samples.iter().copied().sum::<f64>() / sample_count as f64;
        let variance = self
            .samples
            .iter()
            .map(|v| {
                let d = v - mean;
                d * d
            })
            .sum::<f64>()
            / sample_count as f64;
        let jitter =
            self.samples.iter().map(|v| (v - mean).abs()).sum::<f64>() / sample_count as f64;

        EntropyMetrics {
            mean,
            variance,
            jitter,
            sample_count,
        }
    }

    /// Compute a sentry decision from current metrics.
    pub fn decision(&self) -> SentryDecision {
        let m = self.metrics();

        // Not enough data yet – do nothing dramatic.
        if self.total_samples < self.cfg.min_samples {
            return SentryDecision::Keep;
        }

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
