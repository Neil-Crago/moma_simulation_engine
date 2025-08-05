//! # MOMA Simulation Engine
//!
//! A library for creating and running dynamic systems, such as cellular automata,
//! using the MOMA framework as the core update rule.
pub mod automaton;
pub mod grid;
pub mod maze;
pub mod pathfinding;

// Re-export the most important structs for easy access by users of the crate.
pub use grid::{Cell, Grid, Point};
pub use pathfinding::a_star;
pub use automaton::{Moma2dAutomaton, CellularAutomaton};
