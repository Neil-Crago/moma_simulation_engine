//! Represents a single qubit.

use num_complex::Complex;
use std::fmt;

// We'll use 64-bit floats for our calculations.
type F = f64;

/// Represents a single qubit.
/// The state is stored as a 2-element array of complex numbers [alpha, beta],
/// corresponding to the state |ψ⟩ = α|0⟩ + β|1⟩.
#[derive(Debug)] // Add Debug for easy printing during development
pub struct Qubit {
    state: [Complex<F>; 2],
}

impl Qubit {
    /// Creates a new qubit initialized to the |0⟩ state.
    pub fn new() -> Self {
        Self {
            // state[0] = alpha = 1.0 + 0.0i
            // state[1] = beta  = 0.0 + 0.0i
            state: [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)],
        }
    }

    /// Returns the state vector as a slice.
    pub fn get_state_vector(&self) -> &[Complex<F>; 2] {
        &self.state
    }

    /// Applies a quantum gate (represented by a 2x2 matrix) to the qubit.
    /// The new state is calculated by multiplying the gate matrix with the state vector.
    ///
    /// New State = Gate Matrix * Current State
    ///
    /// | α' |   | g00  g01 |   | α |
    /// |    | = |          | * |   |
    /// | β' |   | g10  g11 |   | β |
    pub fn apply_gate(&mut self, gate_matrix: &[[Complex<F>; 2]; 2]) {
        let g00 = gate_matrix[0][0];
        let g01 = gate_matrix[0][1];
        let g10 = gate_matrix[1][0];
        let g11 = gate_matrix[1][1];

        let alpha = self.state[0];
        let beta = self.state[1];

        let new_alpha = g00 * alpha + g01 * beta;
        let new_beta = g10 * alpha + g11 * beta;

        // We should ideally re-normalize here to handle floating point errors,
        // but we can add that later.
        self.state = [new_alpha, new_beta];
    }
}

/// Implement the Display trait for pretty-printing the qubit's state.
impl fmt::Display for Qubit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alpha = self.state[0];
        let beta = self.state[1];
        write!(
            f,
            "({:.3} + {:.3}i)|0⟩ + ({:.3} + {:.3}i)|1⟩",
            alpha.re, alpha.im, beta.re, beta.im
        )
    }
}

// Default implementation will create a qubit in the |0> state.
impl Default for Qubit {
    fn default() -> Self {
        Self::new()
    }
}
