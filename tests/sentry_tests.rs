use entropy_krypton_core::{
    EntropyMetrics, SentryConfig, SentryDecision, SentryEngine, SentrySignals,
};

#[test]
fn low_signals_should_keep() {
    let engine = SentryEngine::with_default_config();

    let metrics = EntropyMetrics::from_samples(&[0.01, 0.0, -0.01, 0.02]);
    let signals = SentrySignals::from_metrics(&metrics, 0.2);

    let decision = engine.decide("job-low", &signals);
    assert_eq!(decision, SentryDecision::Keep);
}

#[test]
fn slightly_high_signals_should_throttle() {
    let mut config = SentryConfig::default();
    config.max_entropy_score = 0.15;
    let engine = SentryEngine::new(config);

    let metrics = EntropyMetrics::from_samples(&[0.5, -0.5, 0.4, -0.4]);
    let signals = SentrySignals::from_metrics(&metrics, 0.5);

    let decision = engine.decide("job-mid", &signals);
    assert_eq!(decision, SentryDecision::Throttle);
}

#[test]
fn extreme_signals_should_kill() {
    let config = SentryConfig {
        max_entropy_score: 0.5,
        max_jitter_score: 0.5,
        max_load_score: 0.8,
        hard_excess_factor: 2.0,
    };
    let engine = SentryEngine::new(config);

    // Deliberately large variance and load.
    let metrics = EntropyMetrics::from_samples(&[10.0, -10.0, 8.0, -8.0]);
    let signals = SentrySignals::from_metrics(&metrics, 2.0);

    let decision = SentryDecision::Kill;
    assert_eq!(engine.decide("job-high", &signals), decision);
}
