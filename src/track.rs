//! Synthetic spiral and helix tracks with known ground truth.
//!
//! A track is a rotation about an axis that wanders. The simulator keeps the
//! true axis and phase, so any estimator from [`moma`] can be scored against the
//! truth rather than judged by eye. Nothing here is random unless asked: equal
//! specs and seeds give identical tracks.

use crate::rng::Rng;
use std::f64::consts::TAU;

/// How the true axis moves, in the plane of the spiral.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wander {
    /// The axis stays at the origin.
    Fixed,
    /// Constant velocity `(vx, vy)` per sample.
    Drift { vx: f64, vy: f64 },
    /// Gaussian random walk with the given step standard deviation.
    Walk { step: f64 },
    /// Circular wobble of the given amplitude and period (in samples).
    Wobble { amplitude: f64, period: f64 },
}

/// How the angle advances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rotation {
    /// Constant angular speed.
    Steady,
    /// Each step's speed is scaled by `1 + jitter * N(0, 1)`, never below 5% of nominal.
    Irregular { jitter: f64 },
    /// The direction flips every `every` samples.
    Reversing { every: usize },
}

/// A 2D track and its ground truth.
#[derive(Debug, Clone)]
pub struct Track2d {
    /// Observed points; dropped samples are `(NaN, NaN)`.
    pub points: Vec<(f64, f64)>,
    /// True axis, one point per sample.
    pub axis: Vec<(f64, f64)>,
    /// True unwrapped phase in radians.
    pub phase: Vec<f64>,
    /// Nominal period in samples.
    pub period: f64,
}

/// A 3D track and its ground truth.
#[derive(Debug, Clone)]
pub struct Track3d {
    /// Observed points; dropped samples are `[NaN; 3]`.
    pub points: Vec<[f64; 3]>,
    /// True axis, one point per sample.
    pub axis: Vec<[f64; 3]>,
    /// True unwrapped phase in radians.
    pub phase: Vec<f64>,
    /// Nominal period in samples.
    pub period: f64,
}

/// Parameters of a simulated spiral. Build with [`SpiralSpec::default`] and the
/// `with_*` methods.
#[derive(Debug, Clone, PartialEq)]
pub struct SpiralSpec {
    pub len: usize,
    /// Nominal samples per turn.
    pub period: f64,
    /// Semi-axis along the rotating x direction.
    pub radius: f64,
    /// Ratio of the other semi-axis to `radius` (1 is a circle).
    pub aspect: f64,
    pub wander: Wander,
    pub rotation: Rotation,
    /// Standard deviation of Gaussian measurement noise.
    pub noise: f64,
    /// Probability that a sample is lost.
    pub dropout: f64,
    pub seed: u64,
}

impl Default for SpiralSpec {
    fn default() -> Self {
        Self {
            len: 1000,
            period: 40.0,
            radius: 1.0,
            aspect: 1.0,
            wander: Wander::Fixed,
            rotation: Rotation::Steady,
            noise: 0.0,
            dropout: 0.0,
            seed: 1,
        }
    }
}

impl SpiralSpec {
    pub fn with_len(mut self, len: usize) -> Self {
        self.len = len;
        self
    }
    pub fn with_period(mut self, period: f64) -> Self {
        self.period = period;
        self
    }
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }
    pub fn with_aspect(mut self, aspect: f64) -> Self {
        self.aspect = aspect;
        self
    }
    pub fn with_wander(mut self, wander: Wander) -> Self {
        self.wander = wander;
        self
    }
    pub fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn with_noise(mut self, noise: f64) -> Self {
        self.noise = noise;
        self
    }
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Checks the parameters, returning the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        validate_common(self.len, self.period, self.radius, self.aspect, self.noise, self.dropout)?;
        validate_rotation(self.rotation)?;
        validate_wander(self.wander)
    }

    /// Generates the track.
    pub fn simulate(&self) -> Result<Track2d, String> {
        self.validate()?;
        let mut rng = Rng::new(self.seed);
        let axis = wander_path(self.wander, self.len, &mut rng);
        let phase = phase_path(self.rotation, self.period, self.len, &mut rng);
        let b = self.radius * self.aspect;
        let points = (0..self.len)
            .map(|i| {
                let x = axis[i].0 + self.radius * phase[i].cos() + self.noise * rng.gaussian();
                let y = axis[i].1 + b * phase[i].sin() + self.noise * rng.gaussian();
                if rng.uniform() < self.dropout { (f64::NAN, f64::NAN) } else { (x, y) }
            })
            .collect();
        Ok(Track2d { points, axis, phase, period: self.period })
    }
}

/// Parameters of a simulated helix: a [`SpiralSpec`] that also advances along z.
#[derive(Debug, Clone, PartialEq)]
pub struct HelixSpec {
    /// The in-plane motion; its `wander` moves the axis in x and y.
    pub spiral: SpiralSpec,
    /// Axis advance along z per sample.
    pub rise: f64,
}

impl Default for HelixSpec {
    fn default() -> Self {
        Self { spiral: SpiralSpec::default(), rise: 0.05 }
    }
}

impl HelixSpec {
    pub fn new(spiral: SpiralSpec, rise: f64) -> Self {
        Self { spiral, rise }
    }

    /// Checks the parameters, returning the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if !self.rise.is_finite() {
            return Err("rise must be finite".into());
        }
        self.spiral.validate()
    }

    /// Generates the track.
    pub fn simulate(&self) -> Result<Track3d, String> {
        self.validate()?;
        let flat = self.spiral.simulate()?;
        let axis: Vec<[f64; 3]> =
            flat.axis.iter().enumerate().map(|(i, a)| [a.0, a.1, self.rise * i as f64]).collect();
        let points = flat
            .points
            .iter()
            .zip(&axis)
            .map(|(p, a)| {
                if p.0.is_finite() && p.1.is_finite() {
                    [p.0, p.1, a[2]]
                } else {
                    [f64::NAN; 3]
                }
            })
            .collect();
        Ok(Track3d { points, axis, phase: flat.phase, period: flat.period })
    }
}

fn validate_common(len: usize, period: f64, radius: f64, aspect: f64, noise: f64, dropout: f64) -> Result<(), String> {
    if len < 2 {
        return Err("len must be at least 2".into());
    }
    if !(period.is_finite() && period > 2.0) {
        return Err("period must be finite and greater than 2 samples".into());
    }
    if !(radius.is_finite() && radius > 0.0) {
        return Err("radius must be finite and positive".into());
    }
    if !(aspect.is_finite() && aspect > 0.0) {
        return Err("aspect must be finite and positive".into());
    }
    if !(noise.is_finite() && noise >= 0.0) {
        return Err("noise must be finite and non-negative".into());
    }
    if !(0.0..1.0).contains(&dropout) {
        return Err("dropout must be in [0, 1)".into());
    }
    Ok(())
}

fn validate_rotation(rotation: Rotation) -> Result<(), String> {
    match rotation {
        Rotation::Irregular { jitter } if !(jitter.is_finite() && jitter >= 0.0) => {
            Err("jitter must be finite and non-negative".into())
        }
        Rotation::Reversing { every: 0 } => Err("reversal interval must be positive".into()),
        _ => Ok(()),
    }
}

fn validate_wander(wander: Wander) -> Result<(), String> {
    let ok = match wander {
        Wander::Fixed => true,
        Wander::Drift { vx, vy } => vx.is_finite() && vy.is_finite(),
        Wander::Walk { step } => step.is_finite() && step >= 0.0,
        Wander::Wobble { amplitude, period } => amplitude.is_finite() && period.is_finite() && period > 0.0,
    };
    if ok { Ok(()) } else { Err("invalid wander parameters".into()) }
}

fn wander_path(wander: Wander, len: usize, rng: &mut Rng) -> Vec<(f64, f64)> {
    let mut pos = (0.0, 0.0);
    (0..len)
        .map(|i| {
            let t = i as f64;
            match wander {
                Wander::Fixed => {}
                Wander::Drift { vx, vy } => pos = (vx * t, vy * t),
                Wander::Walk { step } => {
                    if i > 0 {
                        pos.0 += step * rng.gaussian();
                        pos.1 += step * rng.gaussian();
                    }
                }
                Wander::Wobble { amplitude, period } => {
                    let a = TAU * t / period;
                    pos = (amplitude * a.cos(), amplitude * a.sin());
                }
            }
            pos
        })
        .collect()
}

fn phase_path(rotation: Rotation, period: f64, len: usize, rng: &mut Rng) -> Vec<f64> {
    let w = TAU / period;
    let mut phase = 0.0;
    (0..len)
        .map(|i| {
            if i > 0 {
                phase += match rotation {
                    Rotation::Steady => w,
                    Rotation::Irregular { jitter } => w * (1.0 + jitter * rng.gaussian()).max(0.05),
                    Rotation::Reversing { every } => {
                        if (i / every) % 2 == 0 { w } else { -w }
                    }
                };
            }
            phase
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_track() {
        let spec = SpiralSpec::default()
            .with_wander(Wander::Walk { step: 0.05 })
            .with_noise(0.02)
            .with_dropout(0.1);
        let a = spec.simulate().unwrap();
        let b = spec.simulate().unwrap();
        assert_eq!(a.axis, b.axis);
        assert_eq!(a.phase, b.phase);
        for (p, q) in a.points.iter().zip(&b.points) {
            assert!(p == q || (p.0.is_nan() && q.0.is_nan()));
        }
        let c = spec.clone().with_seed(2).simulate().unwrap();
        assert_ne!(a.axis, c.axis);
    }

    #[test]
    fn noiseless_points_lie_on_the_orbit() {
        let t = SpiralSpec::default().with_radius(2.0).with_aspect(0.5).simulate().unwrap();
        for (p, a) in t.points.iter().zip(&t.axis) {
            let (dx, dy) = ((p.0 - a.0) / 2.0, (p.1 - a.1) / 1.0);
            assert!((dx * dx + dy * dy - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn steady_phase_advances_one_turn_per_period() {
        let t = SpiralSpec::default().with_period(50.0).with_len(101).simulate().unwrap();
        assert!((t.phase[100] - 2.0 * TAU).abs() < 1e-9);
    }

    #[test]
    fn reversing_comes_back() {
        let t = SpiralSpec::default()
            .with_rotation(Rotation::Reversing { every: 40 })
            .with_period(20.0)
            .with_len(81)
            .simulate()
            .unwrap();
        assert!(t.phase[80].abs() < 1e-9);
        assert!(t.phase[40] > t.phase[80]);
    }

    #[test]
    fn dropout_marks_gaps() {
        let t = SpiralSpec::default().with_dropout(0.3).simulate().unwrap();
        let lost = t.points.iter().filter(|p| !p.0.is_finite()).count();
        assert!((200..400).contains(&lost), "lost {lost}");
    }

    #[test]
    fn helix_rises_along_z() {
        let h = HelixSpec::new(SpiralSpec::default().with_len(11), 0.5).simulate().unwrap();
        assert!((h.axis[10][2] - 5.0).abs() < 1e-12);
        assert!((h.points[10][2] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_specs() {
        assert!(SpiralSpec::default().with_len(1).simulate().is_err());
        assert!(SpiralSpec::default().with_period(1.0).simulate().is_err());
        assert!(SpiralSpec::default().with_radius(0.0).simulate().is_err());
        assert!(SpiralSpec::default().with_noise(-1.0).simulate().is_err());
        assert!(SpiralSpec::default().with_dropout(1.0).simulate().is_err());
        assert!(SpiralSpec::default().with_rotation(Rotation::Reversing { every: 0 }).simulate().is_err());
        assert!(SpiralSpec::default().with_wander(Wander::Walk { step: f64::NAN }).simulate().is_err());
        assert!(HelixSpec::new(SpiralSpec::default(), f64::INFINITY).simulate().is_err());
    }
}
