//! # Cellular Automaton Module
//!
//! Provides a 2D cellular automaton that uses a MOMA ring as its update rule.

use moma::core::{MomaRing, OriginStrategy};
use rand::Rng;

/// Represents a 1D Cellular Automaton whose rules are governed by MOMA.
pub struct CellularAutomaton<S: OriginStrategy> {
    /// The current state of all cells.
    state: Vec<u64>,
    /// The width of the automaton.
    width: usize,
    /// The MOMA ring that defines the update rules.
    ring: MomaRing<S>,
}

impl<S: OriginStrategy + Clone> CellularAutomaton<S> {
    /// Creates a new CellularAutomaton with a random initial state.
    ///
    /// # Arguments
    /// * `width` - The number of cells in the automaton.
    /// * `modulus` - The modulus for the MOMA ring. This also defines the max state of a cell.
    /// * `strategy` - The MOMA strategy to use for the update rules.
    pub fn new(width: usize, modulus: u64, strategy: S) -> Self {
        let mut rng = rand::rng();
        let state = (0..width).map(|_| rng.random_range(0..modulus)).collect();

        Self {
            state,
            width,
            ring: MomaRing::new(modulus, strategy),
        }
    }

    /// Advances the simulation by one time step.
    ///
    /// It calculates the next state for each cell based on its current state and the
    /// state of its immediate neighbors, using the MOMA update rule.
    pub fn step(&mut self) {
        let mut next_state = self.state.clone();

        for i in 0..self.width {
            // Get the states of the left, center, and right cells, wrapping around the edges.
            let left = self.state[(i + self.width - 1) % self.width];
            let center = self.state[i];
            let right = self.state[(i + 1) % self.width];

            // The MOMA Update Rule:
            // The "context" for the moving origin is the sum of the neighbors.
            // This simulates an environmental influence on the cell's evolution.
            let context = left.wrapping_add(right);
            let new_value = self.ring.residue(center, context);

            next_state[i] = new_value;
        }

        self.state = next_state;
    }

    /// Renders the current state of the automaton as a string for display.
    ///
    /// It maps each cell's numerical state to a character for visualization.
    pub fn render(&self) -> String {
        self.state
            .iter()
            .map(|&val| {
                // Map the cell's value to a character.
                // This creates a simple grayscale-like visualization.
                match val % 10 {
                    0 => ' ',
                    1 => '.',
                    2 => ':',
                    3 => '-',
                    4 => '=',
                    5 => '+',
                    6 => '*',
                    7 => '#',
                    8 => '%',
                    _ => '@',
                }
            })
            .collect()
    }
}


/// Represents a 2D Cellular Automaton whose rules are governed by MOMA.
pub struct Moma2dAutomaton<S: OriginStrategy> {
    /// The current state of all cells, stored in a flat vector.
    pub state: Vec<u64>,
    /// The width of the automaton grid.
    pub width: usize,
    /// The height of the automaton grid.
    pub height: usize,
    /// The MOMA ring that defines the update rules.
    ring: MomaRing<S>,
}

impl<S: OriginStrategy + Clone> Moma2dAutomaton<S> {
    /// Creates a new 2D Automaton with a random initial state.
    pub fn new(width: usize, height: usize, modulus: u64, strategy: S) -> Self {
        let mut rng = rand::rng();
        let size = width * height;
        let state = (0..size).map(|_| rng.random_range(0..modulus)).collect();

        Self {
            state,
            width,
            height,
            ring: MomaRing::new(modulus, strategy),
        }
    }

    /// Advances the simulation by one time step.
    pub fn step(&mut self) {
        let mut next_state = self.state.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                // Get the sum of the Moore neighborhood (8 neighbors), wrapping around the edges.
                let mut neighbor_sum = 0;
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = (x as isize + dx + self.width as isize) as usize % self.width;
                        let ny = (y as isize + dy + self.height as isize) as usize % self.height;
                        neighbor_sum += self.state[ny * self.width + nx];
                    }
                }

                let current_index = y * self.width + x;
                let center_value = self.state[current_index];

                // The MOMA Update Rule: The cell's next state is a function of its
                // current state and the influence of its neighbors.
                next_state[current_index] = self.ring.residue(center_value, neighbor_sum);
            }
        }
        self.state = next_state;
    }
}
