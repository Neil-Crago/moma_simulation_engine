//! The gates module for qubit simulation

use num_complex::Complex;

// Re-using our type alias for 64-bit floats
type F = f64;

// The Pauli-X (or NOT) gate matrix.
pub static PAULI_X: [[Complex<F>; 2]; 2] = [
    [Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)],
    [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)],
];

// The Pauli-Y gate matrix.
pub static PAULI_Y: [[Complex<F>; 2]; 2] = [
    [Complex::new(0.0, 0.0), Complex::new(0.0, -1.0)], // [0, -i]
    [Complex::new(0.0, 1.0), Complex::new(0.0, 0.0)],  // [i,  0]
];

// The Pauli-Z gate matrix.
pub static PAULI_Z: [[Complex<F>; 2]; 2] = [
    [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)],
    [Complex::new(0.0, 0.0), Complex::new(-1.0, 0.0)],
];
// The Hadamard gate matrix.
pub static HADAMARD: [[Complex<F>; 2]; 2] = [
    [Complex::new(1.0 / std::f64::consts::SQRT_2, 0.0), Complex::new(1.0 / std::f64::consts::SQRT_2, 0.0)],
    [Complex::new(1.0 / std::f64::consts::SQRT_2, 0.0), Complex::new(-1.0 / std::f64::consts::SQRT_2, 0.0)],
];