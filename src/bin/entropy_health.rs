use krypton_entropy_core::{EntropyConfig, EntropyEngine, EntropyMetrics, SentryDecision};
use std::env;

fn decision_str(d: SentryDecision) -> &'static str {
    match d {
        SentryDecision::Keep => "Keep",
        SentryDecision::Throttle => "Throttle",
        SentryDecision::Kill => "Kill",
    }
}

fn parse_samples() -> u64 {
    env::args()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2048)
}

fn main() {
    let samples = parse_samples();
    let cfg = EntropyConfig::default();
    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..samples {
        engine.sample();
    }

    let metrics: EntropyMetrics = engine.metrics();
    let decision = engine.decision();

    println!(
        "{{\"samples\":{},\"mean\":{},\"variance\":{},\"jitter\":{},\"decision\":\"{}\"}}",
        metrics.sample_count,
        metrics.mean,
        metrics.variance,
        metrics.jitter,
        decision_str(decision),
    );
}
