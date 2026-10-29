//! Scoring moma's axis estimators against simulated ground truth.
//!
//! [`Estimator`] wraps each axis method in [`moma::spiral`] behind one interface,
//! [`evaluate`] scores them on a [`Track2d`], and [`consensus`] merges them with
//! [`moma::consensus::consensus_axis_2d`] and scores the result the same way.

use crate::track::Track2d;
use moma::consensus::{AxisConsensus, consensus_axis_2d};
use moma::spiral;

/// One way of estimating a moving axis from a 2D track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Estimator {
    /// Centred moving mean over a fixed window ([`spiral::moving_axis`]).
    Fixed(usize),
    /// Window adapted to the local rotation speed ([`spiral::adaptive_axis`]).
    Adaptive { window: usize, iterations: usize },
    /// Window matched to the dominant period found in `[min_period, max_period]`
    /// ([`spiral::auto_axis`]). Fails if no period is found.
    Auto { min_period: f64, max_period: f64 },
    /// Mean over exactly `period` samples ([`spiral::moving_axis_period`]).
    PeriodMatched(f64),
    /// Moving median ([`spiral::moving_median_axis`]).
    Median(usize),
    /// Centre of the window's coordinate range ([`spiral::moving_midrange_axis`]).
    Midrange { window: usize, trim: f64 },
    /// Window grown until one full turn is seen ([`spiral::coverage_axis`]).
    Coverage { min_window: usize, max_window: usize, trim: f64 },
    /// Joint axis and rotation fit ([`spiral::refine_axis`]).
    Refined { window: usize, iterations: usize },
}

impl Estimator {
    /// Short label for tables.
    pub fn name(&self) -> &'static str {
        match self {
            Estimator::Fixed(_) => "fixed",
            Estimator::Adaptive { .. } => "adaptive",
            Estimator::Auto { .. } => "auto",
            Estimator::PeriodMatched(_) => "period",
            Estimator::Median(_) => "median",
            Estimator::Midrange { .. } => "midrange",
            Estimator::Coverage { .. } => "coverage",
            Estimator::Refined { .. } => "refined",
        }
    }

    /// Estimates the axis. Gaps (non-finite points) are filled first. Returns
    /// `None` when the method cannot produce an axis, or for fewer than two points.
    pub fn estimate(&self, points: &[(f64, f64)]) -> Option<Vec<(f64, f64)>> {
        if points.len() < 2 || points.iter().all(|p| !p.0.is_finite() || !p.1.is_finite()) {
            return None;
        }
        let filled = spiral::fill_gaps(points);
        let axis = match *self {
            Estimator::Fixed(w) => spiral::moving_axis(&filled, w),
            Estimator::Adaptive { window, iterations } => spiral::adaptive_axis(&filled, window, iterations),
            Estimator::Auto { min_period, max_period } => spiral::auto_axis(&filled, min_period, max_period)?.0,
            Estimator::PeriodMatched(p) => spiral::moving_axis_period(&filled, p),
            Estimator::Median(w) => spiral::moving_median_axis(&filled, w),
            Estimator::Midrange { window, trim } => spiral::moving_midrange_axis(&filled, window, trim),
            Estimator::Coverage { min_window, max_window, trim } => {
                spiral::coverage_axis(&filled, min_window, max_window, trim)
            }
            Estimator::Refined { window, iterations } => spiral::refine_axis(&filled, window, iterations).axis,
        };
        (axis.len() == points.len()).then_some(axis)
    }

    /// A reasonable set covering every method for a track of the given nominal
    /// period (samples per turn).
    pub fn suite(period: f64) -> Vec<Estimator> {
        let w = spiral::window_for_period(period);
        vec![
            Estimator::Fixed(w),
            Estimator::Adaptive { window: w, iterations: 2 },
            Estimator::Auto { min_period: period / 3.0, max_period: period * 3.0 },
            Estimator::PeriodMatched(period),
            Estimator::Median(w),
            Estimator::Midrange { window: w, trim: 0.0 },
            Estimator::Coverage { min_window: w, max_window: 4 * w, trim: 0.0 },
            Estimator::Refined { window: w, iterations: 3 },
        ]
    }
}

/// Root-mean-square distance between two axes, ignoring `trim` samples at each
/// end (moving windows are biased where they are clipped). `None` when the axes
/// differ in length, are too short for the trim, or contain non-finite values.
pub fn axis_error(estimate: &[(f64, f64)], truth: &[(f64, f64)], trim: usize) -> Option<f64> {
    if estimate.len() != truth.len() || estimate.len() <= 2 * trim {
        return None;
    }
    let range = trim..estimate.len() - trim;
    let sum: f64 = estimate[range.clone()]
        .iter()
        .zip(&truth[range.clone()])
        .map(|(e, t)| (e.0 - t.0).powi(2) + (e.1 - t.1).powi(2))
        .sum();
    let rms = (sum / range.len() as f64).sqrt();
    rms.is_finite().then_some(rms)
}

/// The score of one estimator on one track.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub name: &'static str,
    /// RMS axis error, or `None` if the estimator failed.
    pub axis_error: Option<f64>,
}

/// Scores each estimator on `track`. The trim is half a nominal period at each end.
pub fn evaluate(track: &Track2d, estimators: &[Estimator]) -> Vec<Report> {
    let trim = trim_for(track);
    estimators
        .iter()
        .map(|e| Report {
            name: e.name(),
            axis_error: e.estimate(&track.points).and_then(|a| axis_error(&a, &track.axis, trim)),
        })
        .collect()
}

/// Merges the estimators that succeed on `track` into one axis and scores it.
/// Returns the consensus (with per-estimator weights, in the order of the
/// estimators that succeeded) and its error. `None` if none succeeds.
pub fn consensus(track: &Track2d, estimators: &[Estimator]) -> Option<(AxisConsensus<2>, Option<f64>)> {
    let estimates: Vec<Vec<(f64, f64)>> = estimators.iter().filter_map(|e| e.estimate(&track.points)).collect();
    let merged = consensus_axis_2d(&estimates, None)?;
    let tuples: Vec<(f64, f64)> = merged.axis.iter().map(|a| (a[0], a[1])).collect();
    let err = axis_error(&tuples, &track.axis, trim_for(track));
    Some((merged, err))
}

fn trim_for(track: &Track2d) -> usize {
    ((track.period / 2.0).ceil() as usize).min(track.points.len().saturating_sub(1) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::{SpiralSpec, Wander};

    #[test]
    fn axis_error_basics() {
        let a = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let b = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        assert_eq!(axis_error(&a, &a, 0), Some(0.0));
        assert!((axis_error(&a, &b, 0).unwrap() - (1.0f64 / 3.0).sqrt()).abs() < 1e-12);
        assert_eq!(axis_error(&a, &b, 1), Some(1.0));
        assert_eq!(axis_error(&a, &b[..2], 0), None);
        assert_eq!(axis_error(&a, &b, 2), None);
    }

    #[test]
    fn fixed_axis_is_recovered_by_every_estimator() {
        let track = SpiralSpec::default().with_len(600).simulate().unwrap();
        for r in evaluate(&track, &Estimator::suite(track.period)) {
            let e = r.axis_error.unwrap_or_else(|| panic!("{} failed", r.name));
            assert!(e < 0.1, "{}: {e}", r.name);
        }
    }

    #[test]
    fn estimators_beat_taking_the_origin_when_the_axis_drifts() {
        let track = SpiralSpec::default()
            .with_len(800)
            .with_wander(Wander::Drift { vx: 0.02, vy: -0.01 })
            .simulate()
            .unwrap();
        let reports = evaluate(&track, &[Estimator::PeriodMatched(40.0), Estimator::Fixed(41)]);
        let naive = axis_error(&vec![(0.0, 0.0); 800], &track.axis, 20).unwrap();
        for r in reports {
            assert!(r.axis_error.unwrap() < naive / 10.0, "{r:?} vs naive {naive}");
        }
    }

    #[test]
    fn period_matched_beats_a_mismatched_window_on_an_ellipse() {
        let track = SpiralSpec::default()
            .with_len(800)
            .with_aspect(0.4)
            .with_period(37.5)
            .with_wander(Wander::Wobble { amplitude: 0.5, period: 400.0 })
            .simulate()
            .unwrap();
        let matched = Estimator::PeriodMatched(37.5).estimate(&track.points).unwrap();
        let mismatched = Estimator::Fixed(25).estimate(&track.points).unwrap();
        let em = axis_error(&matched, &track.axis, 19).unwrap();
        let ex = axis_error(&mismatched, &track.axis, 19).unwrap();
        assert!(em < ex, "matched {em} vs mismatched {ex}");
    }

    #[test]
    fn dropouts_are_tolerated() {
        let track = SpiralSpec::default().with_len(600).with_dropout(0.1).simulate().unwrap();
        let e = Estimator::PeriodMatched(40.0).estimate(&track.points).unwrap();
        assert!(e.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
        assert!(axis_error(&e, &track.axis, 20).unwrap() < 0.2);
    }

    #[test]
    fn consensus_is_no_worse_than_the_worst_member() {
        let track = SpiralSpec::default()
            .with_len(800)
            .with_noise(0.05)
            .with_wander(Wander::Wobble { amplitude: 0.5, period: 400.0 })
            .simulate()
            .unwrap();
        let suite = Estimator::suite(track.period);
        let worst = evaluate(&track, &suite).iter().filter_map(|r| r.axis_error).fold(0.0, f64::max);
        let (merged, err) = consensus(&track, &suite).unwrap();
        assert!((merged.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!(err.unwrap() <= worst + 1e-12);
    }

    #[test]
    fn failures_are_reported_not_hidden() {
        let flat = vec![(f64::NAN, f64::NAN); 50];
        assert!(Estimator::Fixed(5).estimate(&flat).is_none());
        assert!(Estimator::Fixed(5).estimate(&[(0.0, 0.0)]).is_none());
        let track = SpiralSpec::default().with_len(100).simulate().unwrap();
        // A search range with no period in it must not produce an axis.
        let r = evaluate(&track, &[Estimator::Auto { min_period: 500.0, max_period: 600.0 }]);
        assert_eq!(r[0].axis_error, None);
        assert!(consensus(&track, &[Estimator::Auto { min_period: 500.0, max_period: 600.0 }]).is_none());
    }
}
