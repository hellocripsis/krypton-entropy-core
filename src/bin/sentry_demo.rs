use entropy_sentry_core::{EntropyMetrics, SentryConfig, SentryEngine, SentrySignals};

use rand::Rng;

fn main() {
    let config = SentryConfig::default();
    let engine = SentryEngine::new(config);

    let mut rng = rand::thread_rng();

    println!("=== Sentry demo ===");
    for job_index in 0..10 {
        let job_id = format!("job-{}", job_index + 1);

        // Synthetic samples: mean around 0.0, variance roughly controlled by "volatility".
        let volatility = rng.gen_range(0.1..2.5);
        let samples = generate_samples(&mut rng, 32, volatility);

        let metrics = EntropyMetrics::from_samples(&samples);
        let load_score = rng.gen_range(0.2..1.2); // 0.2 = light load, 1.2 = overloaded
        let signals = SentrySignals::from_metrics(&metrics, load_score);

        let decision = engine.decide(&job_id, &signals);

        println!(
            "[{}] decision={:?} entropy_score={:.3} jitter_score={:.3} load_score={:.3}",
            job_id, decision, signals.entropy_score, signals.jitter_score, signals.load_score
        );
    }
}

fn generate_samples<R: Rng>(rng: &mut R, count: usize, volatility: f64) -> Vec<f64> {
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let v: f64 = rng.gen_range(-1.0..1.0) * volatility;
        samples.push(v);
    }
    samples
}
