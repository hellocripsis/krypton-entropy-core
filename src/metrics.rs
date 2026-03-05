/// Aggregate metrics over a sample window.
#[derive(Debug, Clone)]
pub struct EntropyMetrics {
    pub mean: f64,
    pub variance: f64,
    /// Spread measure for the sample window.
    ///
    /// When produced by [`EntropyEngine::metrics`] this is the **standard
    /// deviation** (sqrt of variance), computed in O(1) from running sums.
    ///
    /// When produced by [`EntropyMetrics::from_samples`] this is the **mean
    /// absolute deviation** (MAD), computed in two passes over the slice.
    pub jitter: f64,
    /// Number of samples included in this snapshot.
    pub sample_count: usize,
}

impl EntropyMetrics {
    /// Build metrics from a slice of raw sample values.
    ///
    /// `jitter` here is the **mean absolute deviation** (MAD).  For values
    /// drawn from an [`EntropyEngine`] prefer [`EntropyEngine::metrics`], which
    /// reports standard deviation and is O(1).
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
