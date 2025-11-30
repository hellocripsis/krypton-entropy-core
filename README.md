# krypton-entropy-core

`krypton-entropy-core` is a small Rust library that turns **OS RNG**
samples into entropy metrics and a simple **sentry decision**:

- `Keep` – everything looks normal  
- `Throttle` – stats look odd, slow down / back off  
- `Kill` – stats are bad enough to deny or abort work  

It uses the system RNG (`rand::rngs::OsRng`) as the **only** entropy
source and computes bit-density stats over a stream of `u64` samples.

---

## What it does

For each sample:

1. Draw a `u64` from `OsRng`.
2. Compute `p = bits_set / 64.0` (bit density in `[0.0, 1.0]`).
3. Update rolling metrics:

```rust
pub struct EntropyMetrics {
    pub mean: f64,
    pub variance: f64,
    pub jitter: f64,
    pub sample_count: u64,
}
````

4. Feed those metrics into a sentry policy that returns:

```rust
pub enum SentryDecision {
    Keep,
    Throttle,
    Kill,
}
```

Configs are controlled via:

```rust
pub struct EntropyConfig {
    pub min_samples: u64,
    pub mean_center: f64,
    pub mean_tolerance: f64,
    pub max_jitter: f64,
    pub kill_jitter: f64,
}
```

There are sensible defaults plus an `aggressive()` profile that is more
likely to return `Throttle` / `Kill` when metrics drift.

---

## Quickstart

From this repo:

```bash
git clone git@github.com:hellocripsis/krypton-entropy-core.git
cd krypton-entropy-core

# run tests
cargo test
```

---

## Example usage

```rust
use krypton_entropy_core::{EntropyConfig, EntropyEngine, EntropyMetrics, SentryDecision};

fn main() {
    // pick default or aggressive config
    let cfg = EntropyConfig::default();
    let mut engine = EntropyEngine::with_config(cfg);

    // take some samples from OsRng
    for _ in 0..10_000 {
        engine.sample();
    }

    let metrics: EntropyMetrics = engine.metrics();
    let decision: SentryDecision = engine.decision();

    println!(
        "mean={} var={} jitter={} n={} decision={:?}",
        metrics.mean,
        metrics.variance,
        metrics.jitter,
        metrics.sample_count,
        decision,
    );
}
```

---

## Relationship to other repos

* [`krypton-consumer`](../krypton-consumer) is a tiny CLI that calls this
  library, streams per-iteration stats, and shows `Keep` / `Throttle` /
  `Kill` decisions in real time.

`krypton-entropy-core` is intended as a **clean, self-contained**
portfolio crate: OS RNG in, metrics + decisions out. No custom RNGs, no
simulated decay, and no proprietary entropy cores.

```
::contentReference[oaicite:0]{index=0}
```
