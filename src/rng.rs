//! A small deterministic random number generator (SplitMix64).
//!
//! Simulations here must be reproducible from a seed on every platform and
//! across releases, so the crate carries its own generator instead of depending
//! on `rand`, whose streams may change between versions.

use std::f64::consts::TAU;

/// SplitMix64 generator. Not cryptographic.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a generator; equal seeds give equal streams.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `[0, 1)`.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `[0, n)`. Returns 0 when `n` is 0.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        // Rejection keeps the result unbiased.
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let v = self.next_u64();
            if v < zone {
                return v % n;
            }
        }
    }

    /// Standard normal value (Box-Muller).
    pub fn gaussian(&mut self) -> f64 {
        let u1 = 1.0 - self.uniform();
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn known_first_value() {
        // Reference SplitMix64 output for seed 0.
        assert_eq!(Rng::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn uniform_in_range_and_unbiased_below() {
        let mut r = Rng::new(1);
        for _ in 0..1000 {
            let u = r.uniform();
            assert!((0.0..1.0).contains(&u));
            assert!(r.below(7) < 7);
        }
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn gaussian_moments() {
        let mut r = Rng::new(3);
        let n = 20_000;
        let xs: Vec<f64> = (0..n).map(|_| r.gaussian()).collect();
        let mean = xs.iter().sum::<f64>() / n as f64;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((var - 1.0).abs() < 0.08, "var {var}");
    }
}
