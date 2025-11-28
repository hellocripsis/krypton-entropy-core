use entropy_krypton_core::{SentryConfig, SentryDecision, SentryEngine, SentrySignals};
use rand::Rng;
use std::env;

#[derive(Debug, Clone, Copy)]
enum JobState {
    Pending,
    Running,
    Throttled,
    Killed,
}

#[derive(Debug)]
struct Job {
    id: String,
    state: JobState,
}

fn load_config_from_args() -> SentryConfig {
    // Optional: first CLI arg is a JSON config file path
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        match SentryConfig::from_json_file(path) {
            Ok(cfg) => {
                println!("Loaded SentryConfig from {path}");
                cfg
            }
            Err(err) => {
                eprintln!("Failed to load config from {path}: {err}. Falling back to default.");
                SentryConfig::default()
            }
        }
    } else {
        SentryConfig::default()
    }
}

fn main() {
    println!("=== Krypton Sentry demo (job scheduler simulation) ===");

    let config = load_config_from_args();
    let engine = SentryEngine::new(config);

    // Create 10 fake jobs
    let mut jobs: Vec<Job> = (1..=10)
        .map(|i| Job {
            id: format!("job-{i}"),
            state: JobState::Pending,
        })
        .collect();

    let mut rng = rand::thread_rng();

    // Simulate a few "ticks" of the system
    for tick in 0..5 {
        println!("\n--- Tick {tick} ---");

        for job in &mut jobs {
            // Skip jobs that are already killed
            if matches!(job.state, JobState::Killed) {
                continue;
            }

            // Fake "entropy-style" signal scores for this job at this tick.
            // In a real system, these would come from metrics (latency, errors, queue depth).
            let entropy_score: f64 = rng.gen_range(0.0..1.2);
            let jitter_score: f64 = rng.gen_range(0.0..1.0);
            let load_score: f64 = rng.gen_range(0.0..1.5);

            let signals = SentrySignals::from_raw(entropy_score, jitter_score, load_score);
            let decision = engine.decide(&job.id, &signals);

            // Update job state based on decision
            job.state = match decision {
                SentryDecision::Keep => JobState::Running,
                SentryDecision::Throttle => JobState::Throttled,
                SentryDecision::Kill => JobState::Killed,
            };

            println!(
                "[{}] decision={:?} state={:?} entropy_score={:.3} jitter_score={:.3} load_score={:.3}",
                job.id,
                decision,
                job.state,
                signals.entropy_score,
                signals.jitter_score,
                signals.load_score
            );
        }
    }

    // Final summary
    let mut kept = 0;
    let mut throttled = 0;
    let mut killed = 0;

    for job in &jobs {
        match job.state {
            JobState::Running => kept += 1,
            JobState::Throttled => throttled += 1,
            JobState::Killed => killed += 1,
            JobState::Pending => {}
        }
    }

    println!("\n=== Summary ===");
    println!("jobs kept:      {kept}");
    println!("jobs throttled: {throttled}");
    println!("jobs killed:    {killed}");
}
