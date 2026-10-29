//! Scores every moma axis estimator, and their consensus, on synthetic tracks
//! with known ground truth. Run with `cargo run --release --example axis_bench`.
//!
//! Numbers are RMS axis error in units of the orbit radius (1.0); lower is better.
//! `-` means the estimator failed.

use moma_simulation_engine::{Estimator, Rotation, SpiralSpec, Wander, consensus, evaluate};

fn main() {
    let base = SpiralSpec::default().with_len(1500).with_period(40.0).with_aspect(0.6);
    let wobble = Wander::Wobble { amplitude: 0.8, period: 500.0 };
    let scenarios = [
        ("steady, wobble", base.clone().with_wander(wobble)),
        ("steady, walk", base.clone().with_wander(Wander::Walk { step: 0.03 })),
        ("steady, wobble, noise 0.1", base.clone().with_wander(wobble).with_noise(0.1)),
        ("steady, wobble, 15% dropout", base.clone().with_wander(wobble).with_dropout(0.15)),
        ("irregular, wobble", base.clone().with_wander(wobble).with_rotation(Rotation::Irregular { jitter: 0.5 })),
        ("reversing, wobble", base.clone().with_wander(wobble).with_rotation(Rotation::Reversing { every: 120 })),
    ];

    let suite = Estimator::suite(40.0);
    print!("{:30}", "scenario");
    for e in &suite {
        print!("{:>10}", e.name());
    }
    println!("{:>10}", "consensus");

    for (label, spec) in scenarios {
        let track = spec.simulate().expect("valid spec");
        print!("{label:30}");
        for r in evaluate(&track, &suite) {
            print_cell(r.axis_error);
        }
        print_cell(consensus(&track, &suite).and_then(|(_, e)| e));
        println!();
    }
}

fn print_cell(v: Option<f64>) {
    match v {
        Some(e) => print!("{e:>10.3}"),
        None => print!("{:>10}", "-"),
    }
}
