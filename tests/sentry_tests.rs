use entropy_krypton_core::{SentryConfig, SentryDecision, SentryEngine, SentrySignals};

fn make_engine() -> SentryEngine {
    SentryEngine::new(SentryConfig {
        max_entropy_score: 1.0,
        soft_excess_factor: 1.1, // Keep < 1.1, Throttle between 1.1 and 1.5
        hard_excess_factor: 1.5, // Kill >= 1.5
    })
}

#[test]
fn low_signals_should_keep() {
    let engine = make_engine();

    // stress_score = 0.1 + 0.1 + 0.2 = 0.4  (< soft_threshold 1.1)
    let signals = SentrySignals::from_raw(0.1, 0.1, 0.2);
    let decision = engine.decide("job-low", &signals);

    assert_eq!(decision, SentryDecision::Keep);
}

#[test]
fn slightly_high_signals_should_throttle() {
    let engine = make_engine();

    // stress_score = 0.5 + 0.4 + 0.3 = 1.2  (between 1.1 and 1.5)
    let signals = SentrySignals::from_raw(0.5, 0.4, 0.3);
    let decision = engine.decide("job-mid", &signals);

    assert_eq!(decision, SentryDecision::Throttle);
}

#[test]
fn extreme_signals_should_kill() {
    let engine = make_engine();

    // stress_score = 1.0 + 0.5 + 0.2 = 1.7  (>= 1.5)
    let signals = SentrySignals::from_raw(1.0, 0.5, 0.2);
    let decision = engine.decide("job-high", &signals);

    assert_eq!(decision, SentryDecision::Kill);
}
