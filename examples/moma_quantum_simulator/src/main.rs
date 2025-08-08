use moma_simulation_engine::circuit::QuantumCircuit;
use moma_simulation_engine::gates::{HADAMARD, PAULI_X};

fn main() {
    println!("--- Running Deutsch's Algorithm for f(x) = x (a balanced function) ---");
    let mut circuit = QuantumCircuit::new(2);

    // 1. Initialize to |01⟩. We start at |00⟩ and apply an X-gate to qubit 1.
    circuit.x(1);
    println!("Step 1: Initialized to |01⟩\n{}", circuit);

    // 2. Apply Hadamard to both qubits.
    circuit.h(0)
           .h(1);
    println!("Step 2: After applying H-gates\n{}", circuit);

    // 3. Apply the oracle for f(x) = x, which is CNOT(0, 1).
    circuit.cnot(0, 1);
    println!("Step 3: After applying the Oracle (CNOT)\n{}", circuit);

    // 4. Apply Hadamard to the first qubit.
    circuit.h(0);
    println!("Step 4: After final H-gate\n{}", circuit);

    // 5. Measure the first qubit.
    let outcome = circuit.measure();
    
    println!("--- Result ---");
    println!("Measured state index: {}", outcome);
    
    // The outcome is the full state |xy⟩, we only care about the first qubit, x.
    let first_qubit_measured_value = (outcome >> 1) & 1;

    println!("First qubit measured as: {}", first_qubit_measured_value);

    if first_qubit_measured_value == 0 {
        println!("Conclusion: The function is CONSTANT.");
    } else {
        println!("Conclusion: The function is BALANCED.");
    }
}