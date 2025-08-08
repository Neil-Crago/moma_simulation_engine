//! # MOMA Cellular Automaton Runner
//
// This example creates and runs a 1D cellular automaton powered by the MOMA
// simulation engine. It initializes the automaton with a random state and
// then simulates its evolution over a number of steps, printing each
// generation to the console.

use moma::strategy;
use moma_simulation_engine::CellularAutomaton;
use std::{thread, time};

fn main() {
    println!("--- MOMA-Powered 1D Cellular Automaton ---");

    // --- Simulation Parameters ---
    let width = 100;         // Width of the automaton in cells.
    let steps = 200;         // Number of generations to simulate.
    let modulus = 10;        // The number of states for each cell (0-9).
    let delay_ms = 50;       // Delay between steps for visualization.

    // Choose a MOMA strategy to govern the rules.
    // Different strategies will produce vastly different patterns!
    let strategy = strategy::CompositeMass;

    // --- Initialization ---
    let mut automaton = CellularAutomaton::new(width, modulus, strategy);
    println!("Initial State (Generation 0):");
    println!("{}\n", automaton.render());

    // --- Simulation Loop ---
    for i in 1..=steps {
        automaton.step();
        println!("Generation {}:", i);
        println!("{}", automaton.render());
        thread::sleep(time::Duration::from_millis(delay_ms));
    }

    println!("\n--- Simulation Complete ---");
}
