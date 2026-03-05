# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Changed

- Added `Default` implementation for `EntropyEngine` to satisfy strict clippy
  settings used in CI.
- Added rolling metrics window support with `EntropyConfig.window_size`.
- `EntropyEngine` now tracks total samples separately from rolling-window
  sample count to preserve `min_samples` gating semantics.
- `SentryConfig::from_json_str` and `SentryConfig::from_json_file` now return
  `SentryConfigError` and validate config invariants.
- `SentryEngine::decide` now fails closed (`Kill`) when signals are
  non-finite (`NaN`/`inf`) or derived thresholds are invalid.

### Added

- `SentryConfigError` with `Io`, `Json`, and `Invalid` variants.
- Tests for NaN fail-closed behavior, invalid config rejection, and rolling
  window sample count behavior.

### Documentation

- Updated README API snippets (`sample_count: usize`, `window_size` in
  `EntropyConfig`) and removed broken trailing markup.
