//! Cellular automata whose update rule is a circular consensus on `Z_m`.
//!
//! Each cell holds a residue modulo `m`, a position on a ring. At every step a
//! cell moves to the weighted circular mean of itself and its neighbours
//! ([`moma::consensus::ring_consensus`]). Because the mean is taken on the
//! ring, residues `1` and `m - 1` are close, not far apart. The mean's
//! concentration says how well a neighbourhood agrees, and [`ConsensusAutomaton::order`]
//! summarises that for the whole grid: near 0 for a disordered state, 1 when
//! every cell is at the same residue.
//!
//! Runs are deterministic for a given seed.

use crate::rng::Rng;
use moma::consensus::ring_consensus;

/// Cellular automaton on a graph of cells with a ring-consensus update rule.
#[derive(Debug, Clone)]
pub struct ConsensusAutomaton {
    state: Vec<u64>,
    neighbours: Vec<Vec<usize>>,
    width: usize,
    height: usize,
    modulus: u64,
    self_weight: f64,
    noise: f64,
    rng: Rng,
}

impl ConsensusAutomaton {
    /// A one-dimensional line of `width` cells with wrap-around, each seeing
    /// `radius` cells on either side. Initial residues are uniform random.
    pub fn line(width: usize, modulus: u64, radius: usize, seed: u64) -> Result<Self, String> {
        Self::grid(width, 1, modulus, radius, seed)
    }

    /// A `width` by `height` torus where each cell sees the square of cells
    /// within Chebyshev distance `radius` (`radius` 1 is the Moore neighbourhood).
    /// A height of 1 gives a line.
    pub fn torus(width: usize, height: usize, modulus: u64, radius: usize, seed: u64) -> Result<Self, String> {
        Self::grid(width, height, modulus, radius, seed)
    }

    fn grid(width: usize, height: usize, modulus: u64, radius: usize, seed: u64) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("width and height must be positive".into());
        }
        if modulus < 2 {
            return Err("modulus must be at least 2".into());
        }
        if modulus > 1 << 24 {
            return Err("modulus must be at most 2^24".into());
        }
        if radius == 0 {
            return Err("radius must be at least 1".into());
        }
        let (w, h, r) = (width as isize, height as isize, radius as isize);
        let dy_range = if height == 1 { 0..=0 } else { -r..=r };
        let mut neighbours = Vec::with_capacity(width * height);
        for y in 0..h {
            for x in 0..w {
                let me = (y * w + x) as usize;
                let mut list = Vec::new();
                for dy in dy_range.clone() {
                    for dx in -r..=r {
                        let nx = (x + dx).rem_euclid(w);
                        let ny = (y + dy).rem_euclid(h);
                        let idx = (ny * w + nx) as usize;
                        if idx != me {
                            list.push(idx);
                        }
                    }
                }
                // On small grids the window wraps onto the same cell more than once.
                list.sort_unstable();
                list.dedup();
                neighbours.push(list);
            }
        }
        let mut rng = Rng::new(seed);
        let state = (0..width * height).map(|_| rng.below(modulus)).collect();
        Ok(Self { state, neighbours, width, height, modulus, self_weight: 1.0, noise: 0.0, rng })
    }

    /// Weight of a cell's own residue relative to each neighbour's weight of 1.
    /// Must be finite and non-negative; a larger value makes cells stubborn.
    pub fn with_self_weight(mut self, weight: f64) -> Result<Self, String> {
        if !(weight.is_finite() && weight >= 0.0) {
            return Err("self weight must be finite and non-negative".into());
        }
        self.self_weight = weight;
        Ok(self)
    }

    /// Probability per cell per step of jumping to a uniformly random residue.
    pub fn with_noise(mut self, probability: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&probability) {
            return Err("noise must be in [0, 1]".into());
        }
        self.noise = probability;
        Ok(self)
    }

    /// Replaces the state. Residues are reduced modulo `m`.
    pub fn set_state(&mut self, state: &[u64]) -> Result<(), String> {
        if state.len() != self.state.len() {
            return Err(format!("expected {} cells, got {}", self.state.len(), state.len()));
        }
        self.state = state.iter().map(|&v| v % self.modulus).collect();
        Ok(())
    }

    pub fn state(&self) -> &[u64] {
        &self.state
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn modulus(&self) -> u64 {
        self.modulus
    }

    fn local(&self, i: usize, values: &mut Vec<u64>, weights: &mut Vec<f64>) -> Option<moma::consensus::RingConsensus> {
        values.clear();
        weights.clear();
        values.push(self.state[i]);
        weights.push(self.self_weight);
        for &j in &self.neighbours[i] {
            values.push(self.state[j]);
            weights.push(1.0);
        }
        ring_consensus(values, weights, self.modulus)
    }

    /// Advances one step. All cells update simultaneously from the old state.
    /// A cell whose neighbourhood cancels exactly (no mean direction exists)
    /// keeps its value.
    pub fn step(&mut self) {
        let mut next = self.state.clone();
        let (mut values, mut weights) = (Vec::new(), Vec::new());
        for (i, slot) in next.iter_mut().enumerate() {
            if let Some(c) = self.local(i, &mut values, &mut weights) {
                *slot = c.value;
            }
            if self.noise > 0.0 && self.rng.uniform() < self.noise {
                *slot = self.rng.below(self.modulus);
            }
        }
        self.state = next;
    }

    /// Advances `steps` steps.
    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Global order parameter: the concentration of the circular mean of all
    /// cells. 0 for evenly spread residues, 1 when all cells agree.
    pub fn order(&self) -> f64 {
        let weights = vec![1.0; self.state.len()];
        ring_consensus(&self.state, &weights, self.modulus).map_or(0.0, |c| c.concentration)
    }

    /// Mean over cells of the concentration of each cell's neighbourhood
    /// (itself included, at the configured self weight). High values mean local
    /// agreement even if distant regions disagree.
    pub fn local_agreement(&self) -> f64 {
        let (mut values, mut weights) = (Vec::new(), Vec::new());
        let total: f64 = (0..self.state.len())
            .map(|i| self.local(i, &mut values, &mut weights).map_or(0.0, |c| c.concentration))
            .sum();
        total / self.state.len() as f64
    }

    /// Number of cells at each residue, length `modulus`.
    pub fn histogram(&self) -> Vec<usize> {
        let mut h = vec![0; self.modulus as usize];
        for &v in &self.state {
            h[v as usize] += 1;
        }
        h
    }

    /// Text rendering, one line per row, mapping residues onto ten glyphs by
    /// their position on the ring.
    pub fn render(&self) -> String {
        const GLYPHS: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in self.state.chunks(self.width) {
            for &v in row {
                let g = (v as u128 * 10 / self.modulus as u128) as usize;
                out.push(GLYPHS[g.min(9)]);
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_parameters() {
        assert!(ConsensusAutomaton::line(0, 10, 1, 1).is_err());
        assert!(ConsensusAutomaton::line(10, 1, 1, 1).is_err());
        assert!(ConsensusAutomaton::line(10, u64::MAX, 1, 1).is_err());
        assert!(ConsensusAutomaton::line(10, 10, 0, 1).is_err());
        assert!(ConsensusAutomaton::torus(4, 0, 10, 1, 1).is_err());
        let a = ConsensusAutomaton::line(10, 10, 1, 1).unwrap();
        assert!(a.clone().with_self_weight(-1.0).is_err());
        assert!(a.clone().with_self_weight(f64::NAN).is_err());
        assert!(a.clone().with_noise(1.5).is_err());
        assert!(a.clone().set_state(&[0; 3]).is_err());
    }

    #[test]
    fn same_seed_same_run() {
        let mut a = ConsensusAutomaton::torus(8, 8, 12, 1, 5).unwrap();
        let mut b = ConsensusAutomaton::torus(8, 8, 12, 1, 5).unwrap();
        a.run(20);
        b.run(20);
        assert_eq!(a.state(), b.state());
    }

    #[test]
    fn uniform_state_is_a_fixed_point() {
        let mut a = ConsensusAutomaton::torus(6, 6, 10, 1, 1).unwrap();
        a.set_state(&[7; 36]).unwrap();
        a.run(5);
        assert!(a.state().iter().all(|&v| v == 7));
        assert!((a.order() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn averaging_respects_the_wrap_around() {
        // Residues 9, 1, 9 on a ring of 10 average to about 9.7 (near 0),
        // never to the arithmetic mean 6.33 or to 5.
        let mut a = ConsensusAutomaton::line(5, 10, 1, 1).unwrap();
        a.set_state(&[9, 1, 9, 1, 9]).unwrap();
        a.step();
        for &v in a.state() {
            let dist = v.min(10 - v);
            assert!(dist <= 2, "{v}");
        }
    }

    #[test]
    fn neighbourhood_sizes() {
        let a = ConsensusAutomaton::torus(5, 5, 10, 1, 1).unwrap();
        assert!(a.neighbours.iter().all(|n| n.len() == 8));
        let l = ConsensusAutomaton::line(9, 10, 2, 1).unwrap();
        assert!(l.neighbours.iter().all(|n| n.len() == 4));
        // Wrap-around duplicates on a tiny line collapse to distinct cells.
        let t = ConsensusAutomaton::line(3, 10, 2, 1).unwrap();
        assert!(t.neighbours.iter().all(|n| n.len() == 2));
    }

    #[test]
    fn consensus_dynamics_increase_agreement() {
        let mut a = ConsensusAutomaton::torus(24, 24, 16, 1, 11).unwrap();
        let before = a.local_agreement();
        a.run(30);
        let after = a.local_agreement();
        assert!(after > before, "{before} -> {after}");
        // Domains freeze with defects between them, so agreement stays below 1.
        assert!(after > 0.9, "{after}");
    }

    #[test]
    fn noise_keeps_the_state_from_freezing() {
        let mut quiet = ConsensusAutomaton::torus(16, 16, 16, 1, 3).unwrap();
        let mut noisy = ConsensusAutomaton::torus(16, 16, 16, 1, 3).unwrap().with_noise(0.2).unwrap();
        quiet.run(60);
        noisy.run(60);
        assert!(noisy.local_agreement() < quiet.local_agreement());
    }

    #[test]
    fn histogram_and_render_shapes() {
        let a = ConsensusAutomaton::torus(7, 3, 10, 1, 2).unwrap();
        assert_eq!(a.histogram().iter().sum::<usize>(), 21);
        assert_eq!(a.histogram().len(), 10);
        let r = a.render();
        assert_eq!(r.lines().count(), 3);
        assert!(r.lines().all(|l| l.chars().count() == 7));
    }
}
