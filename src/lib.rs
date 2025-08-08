//! # MOMA Simulation Engine
//!
//! A library for creating and running dynamic systems, such as cellular automata,
//! using the MOMA framework as the core update rule.
pub mod automaton;
pub mod circuit;
pub mod gates;
pub mod qubit;
pub mod grid;
pub mod maze;
pub mod pathfinding;
pub mod network_graph;

// Re-export the most important structs for easy access by users of the crate.

pub use circuit::QuantumCircuit;
pub use gates::{HADAMARD, PAULI_X, PAULI_Y, PAULI_Z};
pub use qubit::Qubit;
pub use grid::{Cell, Grid, Point};
pub use pathfinding::{Node, manhattan_distance, a_star};
pub use automaton::{Moma2dAutomaton, CellularAutomaton};
pub use network_graph::{Graph, Edge};
pub use maze::generate_maze;
