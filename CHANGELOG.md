# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Changed — second hardening pass

- `EntropyEngine::metrics()` is now **O(1)** via incremental sum /
  sum-of-squares tracking (`sum_x`, `sum_x2`) updated on every `sample()`.
  The rolling `VecDeque` is still retained for window eviction, but no longer
  iterated on read paths.
- `jitter` in `EntropyMetrics` produced by `EntropyEngine::metrics()` is now
  **standard deviation** (sqrt of variance) rather than mean absolute
  deviation (MAD).  `EntropyMetrics::from_samples()` still reports MAD; the
  distinction is documented in the `jitter` field doc comment.
- `EntropyConfig` default and aggressive jitter thresholds updated to match
  stddev semantics (`max_jitter: 0.10→0.12`, `kill_jitter: 0.18→0.22`,
  aggressive `0.07→0.09`, `0.14→0.17`).
- `EntropyConfig` now validates all fields via `EntropyConfig::validate()`,
  called automatically from `EntropyEngine::with_config` (panics) and
  `EntropyEngine::try_with_config` (returns `Result`).
- Removed dead `MetricSnapshot` struct from `src/metrics.rs`.
- `SentryConfig` doc updated: collapsed Throttle-band behaviour when
  `soft_excess_factor == hard_excess_factor` is now explicitly documented.
- `SentrySignals` doc updated with a normalization contract — values should
  be in roughly `[0.0, 1.0]` per dimension; raw unbounded values must be
  normalised before use.

### Added — second hardening pass

- `EntropyConfigError` newtype with `Display` + `std::error::Error` impls.
  Re-exported from `lib.rs`.
- `EntropyEngine::try_with_config(cfg)` — fallible constructor returning
  `Result<Self, EntropyConfigError>`.
- `EntropyConfig::validate()` — public method; checks finite values, field
  ordering (`kill_jitter >= max_jitter`), `mean_center` in `(0.0, 1.0)`, and
  `window_size >= 1`.
- `EntropyEngine::reset()` — clears window and running sums while preserving
  config; re-applies the `min_samples` warm-up gate.
- MSRV `rust-version = "1.58"` added to `Cargo.toml`.
- Separate `msrv` CI job pinned to Rust 1.58 added to `ci.yml`.
- `components: rustfmt, clippy` pinned in CI `stable` job.

### Tests — second hardening pass

- `engine_stats_look_reasonable` — assertion now uses `n_samples.min(window_size)`
  instead of asserting exact sample count, making it window-size-invariant.
- New entropy engine tests: `decision_returns_keep_before_min_samples`,
  `total_samples_increments_past_window_size`, `window_size_zero_is_rejected`,
  `inverted_jitter_thresholds_are_rejected`, `validate_rejects_nan_field`,
  `validate_rejects_out_of_range_mean_center`, `reset_clears_all_state`.
- New sentry tests: `score_exactly_at_soft_threshold_should_throttle`,
  `score_exactly_at_hard_threshold_should_kill`,
  `equal_excess_factors_collapse_throttle_band_to_kill`,
  `invalid_config_json_zero_score_should_be_rejected`,
  `validate_called_directly_catches_nan`,
  `validate_called_directly_catches_inf`, extended `nan_signal_should_kill`
  to cover `inf` and `NEG_INFINITY` signals.

### Changed — first hardening pass (prior)

- Added `Default` implementation for `EntropyEngine`.
- Added rolling metrics window support with `EntropyConfig.window_size`.
- `EntropyEngine` tracks `total_samples` separately from window occupancy to
  preserve `min_samples` gating semantics.
- `SentryConfig::from_json_str` and `SentryConfig::from_json_file` return
  `SentryConfigError` and validate config invariants on load.
- `SentryEngine::decide` fails closed (`Kill`) when signals are non-finite or
  derived thresholds are invalid.

### Added — first hardening pass (prior)

- `SentryConfigError` with `Io`, `Json`, and `Invalid` variants.

### Documentation

- Updated README API snippets and migration notes.
- Removed broken trailing markup from README.
