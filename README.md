# entropy-sentry-core

A small Rust library for **metrics-driven job guarding**.

You feed it simple metrics and load signals, and it decides whether a job should be **kept**, **throttled**, or **killed** based on configurable thresholds.

This is a teaching/infra demo crate – not a production policy engine – but it shows how to structure:

- Metrics collection (`EntropyMetrics`)
- Signal extraction (`SentrySignals`)
- Decision logic (`SentryEngine`)

## Quick start

```bash
git clone https://github.com/your-user/entropy-sentry-core.git
cd entropy-sentry-core

# Run tests
cargo test

# Run the demo
cargo run --bin sentry_demo
