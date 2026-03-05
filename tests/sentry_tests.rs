use krypton_entropy_core::{SentryConfig, SentryDecision, SentryEngine, SentrySignals};

fn make_engine() -> SentryEngine {
    SentryEngine::new(SentryConfig {
        max_entropy_score: 1.0,
        soft_excess_factor: 1.1, // Keep < 1.1, Throttle in [1.1, 1.5)
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

    // stress_score = 0.5 + 0.4 + 0.3 = 1.2  (in [1.1, 1.5))
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

#[test]
fn score_exactly_at_soft_threshold_should_throttle() {
    let engine = make_engine();

    // stress_score = 1.1 exactly (== soft_threshold; >= triggers Throttle)
    let signals = SentrySignals::from_raw(0.4, 0.4, 0.3);
    assert_eq!(
        signals.entropy_score + signals.jitter_score + signals.load_score,
        1.1
    );
    let decision = engine.decide("job-soft-exact", &signals);

    assert_eq!(decision, SentryDecision::Throttle);
}

#[test]
fn score_exactly_at_hard_threshold_should_kill() {
    let engine = make_engine();

    // stress_score = 1.5 exactly (== hard_threshold; >= triggers Kill)
    let signals = SentrySignals::from_raw(0.5, 0.5, 0.5);
    assert_eq!(
        signals.entropy_score + signals.jitter_score + signals.load_score,
        1.5
    );
    let decision = engine.decide("job-hard-exact", &signals);

    assert_eq!(decision, SentryDecision::Kill);
}

#[test]
fn nan_signal_should_kill_fail_closed() {
    let engine = make_engine();

    for bad_signals in [
        SentrySignals::from_raw(f64::NAN, 0.1, 0.2),
        SentrySignals::from_raw(0.1, f64::NAN, 0.2),
        SentrySignals::from_raw(0.1, 0.2, f64::NAN),
        SentrySignals::from_raw(f64::INFINITY, 0.1, 0.2),
        SentrySignals::from_raw(f64::NEG_INFINITY, 0.1, 0.2),
    ] {
        let decision = engine.decide("job-bad", &bad_signals);
        assert_eq!(
            decision,
            SentryDecision::Kill,
            "expected Kill for non-finite signal, got {:?}",
            decision
        );
    }
}

#[test]
fn equal_excess_factors_collapse_throttle_band_to_kill() {
    // soft == hard: no Throttle band exists; scores at the threshold go straight to Kill.
    let engine = SentryEngine::new(SentryConfig {
        max_entropy_score: 1.0,
        soft_excess_factor: 1.2,
        hard_excess_factor: 1.2,
    });

    // stress_score well above the shared threshold (1.2); should be Kill, not Throttle,
    // because the Throttle band has zero width when soft == hard.
    let signals = SentrySignals::from_raw(0.5, 0.5, 0.5); // sum = 1.5
    let decision = engine.decide("job-collapsed", &signals);

    assert_eq!(decision, SentryDecision::Kill);
}

#[test]
fn invalid_config_json_inverted_factors_should_be_rejected() {
    let json = r#"{
        "max_entropy_score": 1.0,
        "soft_excess_factor": 1.5,
        "hard_excess_factor": 1.1
    }"#;

    let result = SentryConfig::from_json_str(json);
    assert!(result.is_err(), "inverted factors should fail validation");
}

#[test]
fn invalid_config_json_zero_score_should_be_rejected() {
    let json = r#"{
        "max_entropy_score": 0.0,
        "soft_excess_factor": 1.1,
        "hard_excess_factor": 1.5
    }"#;

    let result = SentryConfig::from_json_str(json);
    assert!(
        result.is_err(),
        "max_entropy_score=0 should fail validation"
    );
}

#[test]
fn validate_called_directly_catches_nan() {
    let cfg = SentryConfig {
        max_entropy_score: f64::NAN,
        soft_excess_factor: 1.1,
        hard_excess_factor: 1.5,
    };
    assert!(cfg.validate().is_err(), "NaN should fail direct validation");
}

#[test]
fn validate_called_directly_catches_inf() {
    let cfg = SentryConfig {
        max_entropy_score: f64::INFINITY,
        soft_excess_factor: 1.1,
        hard_excess_factor: 1.5,
    };
    assert!(cfg.validate().is_err(), "inf should fail direct validation");
}
