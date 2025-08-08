//! MOMA Agent Behavioral Analysis Experiment

//
// This program automates the process of testing and profiling the emergent
// behavior of our MOMA-powered agent. It runs a series of headless simulations,
// each using a different MOMA OriginStrategy, and collects data on the
// resulting path length and geometric complexity (Gowers norm).
//
// The final output is a report comparing the "personality" of the agent
// under each strategic configuration.

use moma::core::{MomaRing, OriginStrategy};
use moma::strategy;
use moma_simulation_engine::automaton::Moma2dAutomaton;
use moma_simulation_engine::grid::Point;
use moma_simulation_engine::pathfinding::{manhattan_distance, Node};
use rustfft::{num_complex::Complex as FftComplex, FftPlanner};
use std::collections::{BinaryHeap, HashMap};
use std::time::Instant;

// --- Experiment Configuration ---
const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const SIMULATION_STEPS: u32 = 200; // Number of paths to calculate for each experiment.
const MODULUS: u64 = 16;

// A struct to hold the results of a single experiment run.
#[derive(Debug)]
struct ExperimentResult {
    name: String,
    samples: u32,
    total_path_length: u64,
    total_gowers_norm: f64,
}

impl ExperimentResult {
    fn new(name: String) -> Self {
        ExperimentResult {
            name,
            samples: 0,
            total_path_length: 0,
            total_gowers_norm: 0.0,
        }
    }

    fn record_sample(&mut self, path_length: u64, gowers_norm: f64) {
        self.samples += 1;
        self.total_path_length += path_length;
        self.total_gowers_norm += gowers_norm;
    }

    fn report(&self) {
        let avg_len = self.total_path_length as f64 / self.samples as f64;
        let avg_norm = self.total_gowers_norm / self.samples as f64;
        println!(
            "| {:<18} | {:<12.2} | {:<13.4} |",
            self.name, avg_len, avg_norm
        );
    }
}

// --- Core Simulation and Analysis Logic (headless) ---

fn main() {
    println!("--- MOMA Agent Behavioral Analysis ---");
    println!(
        "Running {} simulation steps for each strategy...",
        SIMULATION_STEPS
    );

    // Use an enum to represent all possible strategies
    #[derive(Clone)]
    enum StrategyType {
        PrimeGap,
        CompositeMass,
        Fixed(u64),
    }

    impl StrategyType {
        fn name(&self) -> String {
            match self {
                StrategyType::PrimeGap => "PrimeGap".to_string(),
                StrategyType::CompositeMass => "CompositeMass".to_string(),
                StrategyType::Fixed(n) => format!("Fixed({})", n),
            }
        }
    }

    let strategies_to_test: Vec<StrategyType> = vec![
        StrategyType::PrimeGap,
        StrategyType::CompositeMass,
        StrategyType::Fixed(3),
        StrategyType::Fixed(7),
    ];

    let mut results: Vec<ExperimentResult> = Vec::new();
    let total_start_time = Instant::now();

    for strategy in strategies_to_test {
        let start_time = Instant::now();
        println!("\nTesting Strategy: {}...", strategy.name());
        let mut experiment = ExperimentResult::new(strategy.name());

        // Instantiate the correct strategy type
        match strategy.clone() {
            StrategyType::PrimeGap => {
                let mut automaton =
                    Moma2dAutomaton::new(WIDTH as usize, HEIGHT as usize, MODULUS, strategy::PrimeGap);
                let cost_ring = MomaRing::new(MODULUS, strategy::CompositeMass); // Keep cost ring consistent
                let start = Point {
                    x: 10,
                    y: HEIGHT as usize / 2,
                };
                let goal = Point {
                    x: WIDTH as usize - 10,
                    y: HEIGHT as usize / 2,
                };

                for _ in 0..SIMULATION_STEPS {
                    automaton.step();
                    if let Some(path) = a_star_moma_cost(&automaton, &cost_ring, start, goal) {
                        let maze_path_coords: Vec<(i32, i32)> =
                            path.iter().map(|p| (p.x as i32, p.y as i32)).collect();

                        let mut complex_sequence = path_to_complex_sequence_fft(&maze_path_coords);
                        let gowers_norm = calculate_u2_norm_fft(&mut complex_sequence);

                        experiment.record_sample(path.len() as u64, gowers_norm);
                    }
                }
            }
            StrategyType::CompositeMass => {
                let mut automaton =
                    Moma2dAutomaton::new(WIDTH as usize, HEIGHT as usize, MODULUS, strategy::CompositeMass);
                let cost_ring = MomaRing::new(MODULUS, strategy::CompositeMass); // Keep cost ring consistent
                let start = Point {
                    x: 10,
                    y: HEIGHT as usize / 2,
                };
                let goal = Point {
                    x: WIDTH as usize - 10,
                    y: HEIGHT as usize / 2,
                };

                for _ in 0..SIMULATION_STEPS {
                    automaton.step();
                    if let Some(path) = a_star_moma_cost(&automaton, &cost_ring, start, goal) {
                        let maze_path_coords: Vec<(i32, i32)> =
                            path.iter().map(|p| (p.x as i32, p.y as i32)).collect();

                        let mut complex_sequence = path_to_complex_sequence_fft(&maze_path_coords);
                        let gowers_norm = calculate_u2_norm_fft(&mut complex_sequence);

                        experiment.record_sample(path.len() as u64, gowers_norm);
                    }
                }
            }
            StrategyType::Fixed(n) => {
                let mut automaton =
                    Moma2dAutomaton::new(WIDTH as usize, HEIGHT as usize, MODULUS, strategy::Fixed(n));
                let cost_ring = MomaRing::new(MODULUS, strategy::CompositeMass); // Keep cost ring consistent
                let start = Point {
                    x: 10,
                    y: HEIGHT as usize / 2,
                };
                let goal = Point {
                    x: WIDTH as usize - 10,
                    y: HEIGHT as usize / 2,
                };

                for _ in 0..SIMULATION_STEPS {
                    automaton.step();
                    if let Some(path) = a_star_moma_cost(&automaton, &cost_ring, start, goal) {
                        let maze_path_coords: Vec<(i32, i32)> =
                            path.iter().map(|p| (p.x as i32, p.y as i32)).collect();

                        let mut complex_sequence = path_to_complex_sequence_fft(&maze_path_coords);
                        let gowers_norm = calculate_u2_norm_fft(&mut complex_sequence);

                        experiment.record_sample(path.len() as u64, gowers_norm);
                    }
                }
            }
        }
        results.push(experiment);
        println!("  -> Done in {:.2?}", start_time.elapsed());
    }

    // --- Final Report ---
    println!("\n\n--- Experiment Complete ---");
    println!("Total elapsed time: {:.2?}", total_start_time.elapsed());
    println!("\nResults Summary:");
    println!("--------------------------------------------------");
    println!("| Strategy           | Avg. Length  | Avg. Gowers Norm |");
    println!("|--------------------|--------------|------------------|");
    for result in &results {
        result.report();
    }
    println!("--------------------------------------------------");
}

// NOTE: The A*, Gowers, and helper functions are included below and are unchanged.
// They are needed for the headless simulation to run.

fn a_star_moma_cost(
    automaton: &Moma2dAutomaton<impl OriginStrategy>,
    cost_ring: &MomaRing<impl OriginStrategy>,
    start: Point,
    goal: Point,
) -> Option<Vec<Point>> {
    let mut frontier = BinaryHeap::new();
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut cost_so_far: HashMap<Point, u64> = HashMap::new();

    cost_so_far.insert(start, 0);
    frontier.push(Node {
        point: start,
        cost: 0,
        heuristic: manhattan_distance(start, goal),
    });

    while let Some(current) = frontier.pop() {
        if current.point == goal {
            let mut path = vec![goal];
            let mut curr = goal;
            while curr != start {
                curr = came_from[&curr];
                path.push(curr);
            }
            path.reverse();
            return Some(path);
        }

        let neighbors = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(|&(dx, dy)| {
                let nx = current.point.x as isize + dx;
                let ny = current.point.y as isize + dy;
                if nx >= 0
                    && nx < automaton.width as isize
                    && ny >= 0
                    && ny < automaton.height as isize
                {
                    Some(Point {
                        x: nx as usize,
                        y: ny as usize,
                    })
                } else {
                    None
                }
            });

        for next_point in neighbors {
            let current_val = automaton.state[current.point.y * automaton.width + current.point.x];
            let next_val = automaton.state[next_point.y * automaton.width + next_point.x];
            let move_cost = cost_ring.residue(current_val, next_val) + 1;
            let new_cost = cost_so_far[&current.point] + move_cost;

            if !cost_so_far.contains_key(&next_point) || new_cost < cost_so_far[&next_point] {
                cost_so_far.insert(next_point, new_cost);
                let priority = manhattan_distance(next_point, goal);
                frontier.push(Node {
                    point: next_point,
                    cost: new_cost as u32,
                    heuristic: priority,
                });
                came_from.insert(next_point, current.point);
            }
        }
    }
    None
}

fn calculate_u2_norm_fft(sequence: &mut Vec<FftComplex<f64>>) -> f64 {
    let n = sequence.len();
    if n == 0 {
        return 0.0;
    }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(sequence);
    let sum_of_magnitudes_pow4: f64 = sequence
        .iter()
        .map(|c| c.norm_sqr().powi(2))
        .sum();
    (sum_of_magnitudes_pow4 / (n as f64).powi(4)).powf(1.0 / 4.0)
}

fn path_to_complex_sequence_fft(path: &[(i32, i32)]) -> Vec<FftComplex<f64>> {
    if path.len() < 2 {
        return Vec::new();
    }
    let mut complex_sequence: Vec<FftComplex<f64>> = Vec::with_capacity(path.len() - 1);
    for p in 1..path.len() {
        let dx = path[p].0 - path[p - 1].0;
        let dy = path[p].1 - path[p - 1].1;
        let angle = (dy as f64).atan2(dx as f64);
        complex_sequence.push(FftComplex::new(angle.cos(), angle.sin()));
    }
    complex_sequence
}
