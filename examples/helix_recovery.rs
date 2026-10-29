//! Recovers the geometry of a simulated helix with moma's 3D tools and compares
//! it with the truth. Run with `cargo run --release --example helix_recovery`.

use moma::helix::{auto_axis_3d, shared_period_3d, to_cylindrical};
use moma::spiral::radius_spread;
use moma_simulation_engine::{HelixSpec, SpiralSpec, Wander};

fn spec(seed: u64) -> HelixSpec {
    HelixSpec::new(
        SpiralSpec::default()
            .with_len(1200)
            .with_period(36.0)
            .with_radius(2.0)
            .with_wander(Wander::Wobble { amplitude: 0.6, period: 450.0 })
            .with_noise(0.03)
            .with_seed(seed),
        0.05,
    )
}

fn main() {
    let track = spec(1).simulate().expect("valid spec");

    let (axis, period) = auto_axis_3d(&track.points, 12.0, 120.0).expect("a period");
    let (polar, _phase) = to_cylindrical(&track.points, &axis);
    let trim = 20;
    let mid = &polar[trim..polar.len() - trim];
    let mean_radius = mid.iter().map(|p| p.radius).sum::<f64>() / mid.len() as f64;

    println!("period         true {:.2}  found {:.2}", track.period, period);
    println!("radius         true 2.00  found {:.2}  (relative spread {:.3})", mean_radius, radius_spread(mid));

    let tracks: Vec<_> = (0..5).map(|s| spec(s).simulate().expect("valid spec").points).collect();
    let shared = shared_period_3d(&tracks, 12.0, 120.0).expect("a shared period");
    println!("shared period  true {:.2}  found {:.2}  (5 tracks)", track.period, shared);
}
