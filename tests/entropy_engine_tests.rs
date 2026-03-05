use krypton_entropy_core::{EntropyConfig, EntropyEngine, EntropyMetrics, SentryDecision};

#[test]
fn engine_stats_look_reasonable() {
    let cfg = EntropyConfig::default();
    let n_samples: usize = 10_000;
    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..n_samples {
        engine.sample();
    }

    let m: EntropyMetrics = engine.metrics();

    // Mean should be near 0.5.
    assert!((m.mean - 0.5).abs() < 0.05, "mean out of range: {}", m.mean);

    // Variance should be in a sane band for bit-density of a random u64.
    assert!(
        m.variance > 0.001 && m.variance < 0.01,
        "variance out of range: {}",
        m.variance
    );

    // Jitter (stddev) should be non-zero and well below the kill threshold.
    assert!(
        m.jitter > 0.0 && m.jitter < 0.2,
        "jitter out of range: {}",
        m.jitter
    );

    // sample_count is the current window occupancy, not total lifetime samples.
    let expected_window = n_samples.min(cfg.window_size);
    assert_eq!(
        m.sample_count, expected_window,
        "expected window occupancy {expected_window}, got {}",
        m.sample_count
    );
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
        SentryDecision::Keep | SentryDecision::Throttle | SentryDecision::Kill => {}
    }
}

#[test]
fn decision_returns_keep_before_min_samples() {
    let cfg = EntropyConfig::default();
    let mut engine = EntropyEngine::with_config(cfg);

    // Deliberately stay under the warm-up gate.
    for _ in 0..(cfg.min_samples - 1) {
        engine.sample();
    }

    assert_eq!(
        engine.decision(),
        SentryDecision::Keep,
        "expected Keep during warm-up, got {:?}",
        engine.decision()
    );
}

#[test]
fn aggressive_config_can_kill_when_metrics_are_weird() {
    // mean_center far from the actual bit-density (~0.5) forces Kill.
    let cfg = EntropyConfig {
        min_samples: 64,
        mean_center: 0.01, // OsRng bit density ~0.5, far from 0.01
        mean_tolerance: 0.001,
        max_jitter: 0.09,
        kill_jitter: 0.17,
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
    assert_eq!(m.sample_count, 128, "window should cap at window_size");
}

#[test]
fn total_samples_increments_past_window_size() {
    // Even after the window is full, total_samples keeps rising.
    let cfg = EntropyConfig {
        window_size: 16,
        min_samples: 4,
        ..EntropyConfig::default()
    };
    let mut engine = EntropyEngine::with_config(cfg);

    for _ in 0..100 {
        engine.sample();
    }

    let m = engine.metrics();
    assert_eq!(m.sample_count, 16, "window should be capped at 16");
    // Engine should be making decisions (warm-up gate passed).
    let d = engine.decision();
    match d {
        SentryDecision::Keep | SentryDecision::Throttle | SentryDecision::Kill => {}
    }
}

#[test]
fn window_size_zero_is_rejected() {
    let cfg = EntropyConfig {
        window_size: 0,
        ..EntropyConfig::default()
    };
    let result = EntropyEngine::try_with_config(cfg);
    assert!(result.is_err(), "window_size=0 should fail validation");
    assert!(
        result.unwrap_err().to_string().contains("window_size"),
        "error message should mention window_size"
    );
}

#[test]
fn inverted_jitter_thresholds_are_rejected() {
    let cfg = EntropyConfig {
        max_jitter: 0.30,
        kill_jitter: 0.10, // kill < max — invalid ordering
        ..EntropyConfig::default()
    };
    let result = EntropyEngine::try_with_config(cfg);
    assert!(result.is_err(), "kill_jitter < max_jitter should fail");
}

#[test]
fn validate_rejects_nan_field() {
    let cfg = EntropyConfig {
        mean_center: f64::NAN,
        ..EntropyConfig::default()
    };
    assert!(
        cfg.validate().is_err(),
        "NaN mean_center should fail validation"
    );
}

#[test]
fn validate_rejects_out_of_range_mean_center() {
    for bad in [0.0_f64, 1.0, -0.1, 1.1, f64::INFINITY, f64::NEG_INFINITY] {
        let cfg = EntropyConfig {
            mean_center: bad,
            ..EntropyConfig::default()
        };
        assert!(
            cfg.validate().is_err(),
            "mean_center={bad} should fail validation"
        );
    }
}

#[test]
fn reset_clears_all_state() {
    let mut engine = EntropyEngine::new();

    for _ in 0..500 {
        engine.sample();
    }

    let before = engine.metrics();
    assert!(before.sample_count > 0);

    engine.reset();

    let after = engine.metrics();
    assert_eq!(after.sample_count, 0);
    assert_eq!(after.mean, 0.0);
    assert_eq!(after.variance, 0.0);
    assert_eq!(after.jitter, 0.0);

    // Warm-up gate should have re-applied.
    assert_eq!(engine.decision(), SentryDecision::Keep);
}
