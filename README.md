# entropy-krypton-core

A small Rust library for **metrics-driven job guarding**.

You feed it simple metrics and load signals, and it decides whether a job should be **kept**, **throttled**, or **killed** based on configurable thresholds.

This is a teaching/infra demo crate – not a production policy engine – but it shows how to structure:

- Metrics collection (`EntropyMetrics`)
- Signal extraction (`KryptonSignals`)
- Decision logic (`KryptonEngine`)

## Quick start

```bash
git clone https://github.com/your-user/entropy-krypton-core.git
cd entropy-krypton-core

# Run tests
cargo test

# Run the demo
cargo run --bin krypton_demo
