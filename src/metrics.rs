/// A single sampled value with a logical timestamp.
/// In a real system this might be time-based; here it's just a counter.
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub tick: u64,
    pub value: f64,
}

/// Aggregate metrics over a short window of samples.
#[derive(Debug, Clone)]
pub struct EntropyMetrics {
    pub mean: f64,
    pub variance: f64,
    /// A simple "jitter" measure: average absolute deviation from the mean.
    pub jitter: f64,
    /// Number of samples included.
    pub sample_count: usize,
}

impl EntropyMetrics {
    /// Build metrics from a slice of raw sample values.
    pub fn from_samples(samples: &[f64]) -> Self {
        let sample_count = samples.len();
        if sample_count == 0 {
            return Self {
                mean: 0.0,
                variance: 0.0,
                jitter: 0.0,
                sample_count: 0,
            };
        }

        let mean = samples.iter().copied().sum::<f64>() / sample_count as f64;

        let variance = samples
            .iter()
            .map(|v| {
                let d = v - mean;
                d * d
            })
            .sum::<f64>()
            / sample_count as f64;

        let jitter = samples.iter().map(|v| (v - mean).abs()).sum::<f64>() / sample_count as f64;

        Self {
            mean,
            variance,
            jitter,
            sample_count,
        }
    }
}
