//! # MOMA Network Flow Manager
//
// This program demonstrates the full MOMA-Gowers feedback loop applied to a
// network flow problem. It creates a simple network, then uses a "Strategist"
// loop to continually adjust the network's costs to guide the flow pattern
// towards a target level of structural complexity (Gowers norm).


use moma_simulation_engine::network_graph::Graph;
use moma_simulation_engine::grid::Point;
use rustfft::{num_complex::Complex as FftComplex, FftPlanner};
use std::collections::BTreeMap;

// --- Simulation Parameters ---
const SIMULATION_STEPS: u32 = 10;

fn path_to_complex_sequence_fft(path: &[Point]) -> Vec<FftComplex<f64>> {
    if path.len() < 2 { return Vec::new(); }
    let mut complex_sequence = Vec::new();
    for i in 1..path.len() {
        let p1 = path[i-1];
        let p2 = path[i];
        let dx = p2.x as i64 - p1.x as i64;
        let dy = p2.y as i64 - p1.y as i64;
        let angle = (dy as f64).atan2(dx as f64);
        complex_sequence.push(FftComplex::new(angle.cos(), angle.sin()));
    }
    complex_sequence
}

fn main() {
    println!("--- MOMA Network Flow Manager ---");
    
    // --- Controller Tuning ---
    const TARGET_GOWERS_NORM: f64 = 0.85;    
    const COST_ADJUSTMENT_GAIN: f64 = 50.0; // Changed from 5.0
    const COST_DECAY_RATE: f64 = 0.95; // Decay rate is now more effective

    let source = Point { x: 0, y: 1 };
    let sink = Point { x: 3, y: 1 };
    let mut graph = create_diamond_graph(source, sink);

    for i in 0..SIMULATION_STEPS {
        println!("\n--- Step {} ---", i + 1);

        // --- Apply Cost Decay ---
        for edges in graph.adj.values_mut() {
            for edge in edges {
                // Cost decays towards the base cost of 1.0
                edge.cost = 1.0 + (edge.cost - 1.0) * COST_DECAY_RATE;
            }
        }

        let (flow_this_step, path_opt) = graph.route_cheapest_path();
        println!("  - Flow Routed This Step: {}", flow_this_step);

        if let Some(path) = path_opt {
            let mut complex_sequence = path_to_complex_sequence_fft(&path);
            let gowers_norm = calculate_u2_norm_fft(&mut complex_sequence);
            println!("  - Gowers Norm of Path: {:.4}", gowers_norm);

            let error = gowers_norm - TARGET_GOWERS_NORM;
            println!("  - Norm Error: {:.4}", error);

            if error > 0.0 {
                // The crucial fix: NO integer cast.
                let adjustment = error * COST_ADJUSTMENT_GAIN; 
                println!("  - Policy: Path is too simple. Applying cost penalty of {:.3}.", adjustment);
                
                let first_edge_from = path[0];
                let first_edge_to = path[1];
                if let Some(edge) = graph.adj.get_mut(&first_edge_from).unwrap().iter_mut().find(|e| e.to == first_edge_to) {
                    edge.cost += adjustment;
                    println!("  - Action: Cost of edge {:?} -> {:?} is now {:.3}", first_edge_from, first_edge_to, edge.cost);
                }
            } else {
                println!("  - Policy: Path complexity is on target. STABILIZING.");
            }
        }
        
        for edges in graph.adj.values_mut() {
            for edge in edges { edge.flow = 0; }
        }
    }
    println!("\n--- Simulation Complete ---");
}

// ... the create_diamond_graph function also needs to use f64 for costs ...
fn create_diamond_graph(source: Point, sink: Point) -> Graph {
    let mut graph = Graph::new(source, sink);
    let node_a = Point { x: 1, y: 0 };
    let node_b = Point { x: 1, y: 2 };
    let node_c = Point { x: 2, y: 2 };

    // Costs are now f64
    graph.add_edge(source, node_a, 10, 1.0);
    graph.add_edge(node_a, sink, 10, 1.0);
    graph.add_edge(source, node_b, 7, 1.0);
    graph.add_edge(node_b, node_c, 7, 1.0);
    graph.add_edge(node_c, sink, 7, 1.0);

    graph
}
/// Converts the graph's flow values into a canonical, sorted sequence for analysis.
fn _flow_to_sequence(graph: &Graph) -> Vec<f64> {
    // BTreeMap gives us a sorted, canonical ordering of nodes.
    let sorted_adj: BTreeMap<_, _> = graph.adj.iter().collect();
    let mut sequence = Vec::new();

    for (_from_node, edges) in sorted_adj {
        // Sort edges by destination to ensure consistent order
        let mut sorted_edges = edges.clone();
        sorted_edges.sort_by_key(|e| (e.to.x, e.to.y));
        for edge in sorted_edges {
            sequence.push(edge.flow as f64);
        }
    }
    sequence
}

// --- Gowers Norm Calculation Functions (Unchanged) ---
fn calculate_u2_norm_fft(sequence: &mut Vec<FftComplex<f64>>) -> f64 {
    let n = sequence.len();
    if n == 0 { return 0.0; }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(sequence);
    let sum_of_magnitudes_pow4: f64 = sequence.iter().map(|c| c.norm_sqr().powi(2)).sum();
    (sum_of_magnitudes_pow4 / (n as f64).powi(4)).powf(1.0 / 4.0)
}

fn _values_to_complex_sequence_fft(values: &[f64]) -> Vec<FftComplex<f64>> {
    // This function is now more generic: it converts any sequence of f64 values.
    // We'll normalize by the max value to keep numbers between 0 and 1.
    let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max_val == 0.0 { return vec![FftComplex::new(0.0, 0.0); values.len()]; }

    values.iter().map(|&v| {
        let normalized = v / max_val;
        // Map the normalized value to a point on the complex unit circle.
        let angle = normalized * 2.0 * std::f64::consts::PI;
        FftComplex::new(angle.cos(), angle.sin())
    }).collect()
}