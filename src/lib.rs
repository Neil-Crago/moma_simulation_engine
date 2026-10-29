//! # MOMA Simulation Engine
//!
//! Companion to [`moma`]. It generates synthetic tracks with known ground truth
//! and uses them to test and compare moma's analysis tools.
//!
//! * [`track`]: spirals and helices around a wandering axis, with steady,
//!   irregular or reversing rotation, noise and dropped samples.
//! * [`evaluation`]: every axis estimator in [`moma::spiral`] behind one interface,
//!   scored against the true axis, and merged with moma's barycentric consensus.
//! * [`automaton`]: a cellular automaton whose update rule is moma's circular
//!   consensus on `Z_m`.
//! * [`rng`]: a seedable generator so every run is reproducible.
//!
//! ```
//! use moma_simulation_engine::{Estimator, SpiralSpec, Wander, evaluate};
//!
//! let track = SpiralSpec::default()
//!     .with_wander(Wander::Drift { vx: 0.02, vy: 0.0 })
//!     .with_noise(0.02)
//!     .simulate()
//!     .unwrap();
//! for report in evaluate(&track, &Estimator::suite(track.period)) {
//!     println!("{:10} {:?}", report.name, report.axis_error);
//! }
//! ```

pub mod automaton;
pub mod evaluation;
pub mod rng;
pub mod track;

pub use automaton::ConsensusAutomaton;
pub use evaluation::{Estimator, Report, axis_error, consensus, evaluate};
pub use rng::Rng;
pub use track::{HelixSpec, Rotation, SpiralSpec, Track2d, Track3d, Wander};
