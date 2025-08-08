//! Represents a quantum circuit with multiple qubits.
use std::fmt;
use num_complex::Complex;
use rand::Rng;
use crate::gates;

// Re-using our type alias for 64-bit floats
type F = f64;

pub struct QuantumCircuit {
    num_qubits: usize,
    state_vector: Vec<Complex<F>>,
}

impl QuantumCircuit {
    /// Creates a new quantum circuit with a specific number of qubits.
    /// The circuit is initialized in the all-|0⟩ state (e.g., |00...0⟩).
    pub fn new(num_qubits: usize) -> Self {
        // The size of the state vector is 2^n
        let vector_size = 1 << num_qubits;
        let mut state_vector = vec![Complex::new(0.0, 0.0); vector_size];

        // Initialize to the |00...0⟩ state. This state has an amplitude of 1
        // at index 0 and 0 everywhere else.
        state_vector[0] = Complex::new(1.0, 0.0);

        Self {
            num_qubits,
            state_vector,
        }
    }

     /// Applies a Hadamard gate to the target qubit.
    pub fn h(&mut self, target_qubit: usize) -> &mut Self {
        self.apply_single_qubit_gate(target_qubit, &gates::HADAMARD);
        self
    }

    /// Applies a Pauli-X (NOT) gate to the target qubit.
    pub fn x(&mut self, target_qubit: usize) -> &mut Self {
        self.apply_single_qubit_gate(target_qubit, &gates::PAULI_X);
        self
    }
    
    /// Applies a Pauli-Y gate to the target qubit.
    pub fn y(&mut self, target_qubit: usize) -> &mut Self {
        self.apply_single_qubit_gate(target_qubit, &gates::PAULI_Y);
        self
    }

    /// Applies a Pauli-Z gate to the target qubit.
    pub fn z(&mut self, target_qubit: usize) -> &mut Self {
        self.apply_single_qubit_gate(target_qubit, &gates::PAULI_Z);
        self
    }

    /// Applies a CNOT gate.
    pub fn cnot(&mut self, control_qubit: usize, target_qubit: usize) -> &mut Self {
        self.apply_cnot_gate(control_qubit, target_qubit);
        self
    }

/// Applies a single-qubit gate to a specific target qubit in the circuit.
fn apply_single_qubit_gate(&mut self, target_qubit: usize, gate_matrix: &[[Complex<F>; 2]; 2]) {
    // The "stride" is the distance between the two amplitudes we need to modify.
    // For target_qubit 0, stride is 1 (|00⟩ vs |01⟩).
    // For target_qubit 1, stride is 2 (|00⟩ vs |10⟩).
    let stride = 1 << target_qubit;
    let g00 = gate_matrix[0][0];
    let g01 = gate_matrix[0][1];
    let g10 = gate_matrix[1][0];
    let g11 = gate_matrix[1][1];

    // We iterate through the state vector in chunks.
    for i in (0..self.state_vector.len()).step_by(stride * 2) {
        // For each chunk, we apply the gate to pairs of elements.
        for j in i..(i + stride) {
            // These are the two amplitudes that will be mixed by the gate.
            let amplitude0 = self.state_vector[j];
            let amplitude1 = self.state_vector[j + stride];

            // Apply the 2x2 matrix to the pair of amplitudes.
            self.state_vector[j]          = g00 * amplitude0 + g01 * amplitude1;
            self.state_vector[j + stride] = g10 * amplitude0 + g11 * amplitude1;
        }
    }
}

/// Applies a CNOT gate to the circuit.
fn apply_cnot_gate(&mut self, control_qubit: usize, target_qubit: usize) {
    let control_mask = 1 << control_qubit;
    let target_mask = 1 << target_qubit;

    // Iterate through all state vector indices.
    for i in 0..self.state_vector.len() {
        // Check if the control bit is 1 for the current basis state |i⟩.
        if (i & control_mask) != 0 {
            // If the control bit is 1, we swap the amplitudes of the two
            // states that differ only by the target bit.
            // `j` is the index of the other state in the pair.
            let j = i ^ target_mask; // XOR flips the target bit.
            
            // To avoid swapping twice, we only perform the swap
            // when i is the smaller of the two indices.
            if i < j {
                self.state_vector.swap(i, j);
            }
        }
    }
}

/// Measures the entire quantum circuit.
/// Returns the classical outcome as an integer.
pub fn measure(&mut self) -> usize {
    // 1. Get a random number generator.
    let mut rng = rand::rng();
    // Generate a random float between 0.0 and 1.0.
    let random_sample: f64 = rng.random();

    // 2. Calculate the cumulative probability distribution.
    let mut cumulative_prob = 0.0;
    for (i, amplitude) in self.state_vector.iter().enumerate() {
        // The probability is the squared magnitude of the amplitude.
        let probability = amplitude.norm_sqr();
        cumulative_prob += probability;

        // 3. Find the outcome.
        if random_sample < cumulative_prob {
            let measured_index = i;

            // 4. Collapse the wave function.
            // Set all amplitudes to zero...
            self.state_vector.fill(Complex::new(0.0, 0.0));
            // ...except for the one we measured, which is now 1.
            self.state_vector[measured_index] = Complex::new(1.0, 0.0);

            return measured_index;
        }
    }
    // Fallback in case of floating point errors, should not be reached.
    self.state_vector.len() - 1
}
}


impl fmt::Display for QuantumCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, amplitude) in self.state_vector.iter().enumerate() {
            if amplitude.norm_sqr() > 1e-10 { // Only print states with significant amplitude
                writeln!(
                    f,
                    "|{:0width$b}⟩ : {:.3} + {:.3}i",
                    i,
                    amplitude.re,
                    amplitude.im,
                    width = self.num_qubits
                )?;
            }
        }
        Ok(())
    }
}