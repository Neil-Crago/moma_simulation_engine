# Moma Quantum Simulator

A simple, educational quantum circuit simulator built in Rust from first principles. This project demonstrates the core concepts of quantum computing, including superposition, entanglement, and measurement, through a clean, step-by-step implementation.

-----

## Features

  * **Multi-Qubit Simulation**: Simulates quantum systems with n-qubits using a state vector representation.
  * **Fluent API**: A clean, chainable API for building quantum circuits intuitively.
  * **Core Quantum Gates**:
      * Pauli Gates (X, Y, Z)
      * Hadamard Gate
      * Controlled-NOT (CNOT) Gate
  * **Quantum Phenomena**:
      * **Superposition**: Create superpositions with the Hadamard gate.
      * **Entanglement**: Generate a Bell state using H + CNOT gates.
      * **Measurement**: Probabilistic measurement based on the Born rule, with state collapse.
  * **Algorithm Implementation**: Includes a working example of **Deutsch's Algorithm** to demonstrate quantum speedup.

-----

## Requirements

  * **Rust**: The project is built using the Rust programming language. You can install it from [rust-lang.org](https://www.rust-lang.org/).

-----

## Getting Started

1.  **Clone the repository:**

    ```sh
    git clone https://github.com/neil-Crago/moma_simulation_engine.git
    ```

2.  **Navigate to the directory:**

    ```sh
    cd moma_simulation_engine
    ```

3.  **Build and run the project:**

    ```sh
    # To run the example in main.rs
    cargo run

    # To build an optimized release version
    cargo build --release
    ```

-----

## Code Structure

The project is organized into three main files for clarity:

  * `src/main.rs`: The main entry point of the application. Contains examples for running circuits and algorithms like Deutsch's Algorithm.
  * `src/circuit.rs`: Defines the `QuantumCircuit` struct, which holds the state vector and contains all the core logic for applying gates and performing measurements. This is the heart of the simulator.
  * `src/gates.rs`: Contains the static matrix definitions for all implemented quantum gates (Pauli-X, Y, Z, Hadamard, etc.).

-----

## Example Usage

Building a circuit is simple and readable thanks to the fluent API. Here is how to create a 2-qubit entangled Bell state:

```rust
use circuit::QuantumCircuit;

fn main() {
    // Initialize a 2-qubit circuit in the |00⟩ state
    let mut circuit = QuantumCircuit::new(2);

    // Apply a Hadamard gate to qubit 0, then a CNOT with control=0 and target=1
    circuit.h(0)
           .cnot(0, 1);

    // Print the final state vector
    println!("Entangled Bell state:\n{}", circuit);
    // Expected output:
    // |00⟩ : 0.707 + 0.000i
    // |11⟩ : 0.707 + 0.000i
}
```

-----

## Roadmap

This project is a work in progress. Future enhancements may include:

  * **Expanded Gate Set**: Add more standard gates like S, T, and other controlled gates (e.g., Controlled-Z).
  * **More Quantum Algorithms**: Implement other foundational algorithms like Deutsch-Jozsa, Simon's Algorithm, or Grover's Search.
  * **Error Handling**: Add robust error handling for invalid inputs, such as incorrect qubit indices.
  * **Noise Models**: Introduce simple noise models to simulate the behavior of real, non-ideal quantum hardware.

-----

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.