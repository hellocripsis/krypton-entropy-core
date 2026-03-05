use krypton_entropy_core::{EntropyConfig, EntropyEngine, EntropyMetrics, SentryDecision};

#[test]
fn engine_stats_look_reasonable() {
    let cfg = EntropyConfig::default();
    let mut engine = EntropyEngine::with_config(cfg);

    let samples = 10_000_u64;
    for _ in 0..samples {
        engine.sample();
    }

    let m: EntropyMetrics = engine.metrics();

    // Mean should be near 0.5
    assert!((m.mean - 0.5).abs() < 0.05, "mean out of range: {}", m.mean);

    // Variance should be in a sane band for bit-density of a random u64
    assert!(
        m.variance > 0.001 && m.variance < 0.01,
        "variance out of range: {}",
        m.variance
    );

    // Jitter should be non-crazy and we should have all samples counted.
    assert!(
        m.jitter > 0.0 && m.jitter < 0.2,
        "jitter out of range: {}",
        m.jitter
    );
    assert_eq!(m.sample_count as u64, samples);
}

#[test]
fn default_config_decision_does_not_panic() {
    let cfg = EntropyConfig::default();
    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..(cfg.min_samples + 512) {
        engine.sample();
    }

    let d: SentryDecision = engine.decision();
    match d {
        SentryDecision::Keep | SentryDecision::Throttle | SentryDecision::Kill => {} // ok
    }
}

#[test]
fn aggressive_config_can_kill_when_metrics_are_weird() {
    // Deliberately insane config to force Kill once we have enough samples.
    let cfg = EntropyConfig {
        min_samples: 64,
        mean_center: 0.0, // totally wrong on purpose
        mean_tolerance: 0.0001,
        max_jitter: 0.0,
        kill_jitter: 0.0,
        window_size: 1024,
    };

    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..(cfg.min_samples + 512) {
        engine.sample();
    }

    let d = engine.decision();
    assert_eq!(
        d,
        SentryDecision::Kill,
        "expected Kill under hostile config, got {:?}",
        d
    );
}

#[test]
fn metrics_use_rolling_window_size() {
    let cfg = EntropyConfig {
        window_size: 128,
        ..EntropyConfig::default()
    };
    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..10_000 {
        engine.sample();
    }

    let m = engine.metrics();
    assert_eq!(m.sample_count, 128);
}
