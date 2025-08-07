//! # MOMA Network Flow Manager
//!
//! This program demonstrates the full MOMA-Gowers feedback loop applied to a
//! network flow problem. It creates a simple network, then uses a "Strategist"
//! loop to continually adjust the network's costs to guide the flow pattern
//! towards a target level of structural complexity (Gowers norm).


use moma_simulation_engine::network_graph::{Graph, Point};
use rustfft::{num_complex::Complex as FftComplex, FftPlanner};
use std::collections::BTreeMap;

// --- Simulation Parameters ---
const SIMULATION_STEPS: u32 = 10;
const TARGET_GOWERS_NORM: f64 = 0.5; // Target: A moderately complex flow pattern.
const COST_ADJUSTMENT_FACTOR: u64 = 5; // How much to penalize the busiest edge.

/// Creates a simple "diamond" graph for testing the flow manager.
/// This graph has two main paths from source to sink:
/// 1. A direct, high-capacity path (S -> A -> D).
/// 2. A longer, lower-capacity detour (S -> B -> C -> D).
fn create_diamond_graph(source: Point, sink: Point) -> Graph {
    let mut graph = Graph::new(source, sink);
    let node_a = Point { x: 1, y: 0 };
    let node_b = Point { x: 1, y: 2 };
    let node_c = Point { x: 2, y: 2 };

    // High-capacity direct path
    graph.add_edge(source, node_a, 10, 1); // from, to, capacity, cost
    graph.add_edge(node_a, sink, 10, 1);

    // Lower-capacity detour path
    graph.add_edge(source, node_b, 7, 1);
    graph.add_edge(node_b, node_c, 7, 1);
    graph.add_edge(node_c, sink, 7, 1);

    graph
}

fn main() {
    println!("--- MOMA Network Flow Manager ---");

    let source = Point { x: 0, y: 1 };
    let sink = Point { x: 3, y: 1 };
    let mut graph = create_diamond_graph(source, sink);

    for i in 0..SIMULATION_STEPS {
        println!("\n--- Step {} ---", i + 1);

        // 1. TACTICIAN: Run the max-flow algorithm based on current costs.
        let total_flow = graph.edmonds_karp();
        println!("  - Max Flow Calculated: {}", total_flow);

        // 2. OBSERVE: Convert the flow distribution into a canonical sequence.
        let flow_sequence = flow_to_sequence(&graph);

        // 3. ORIENT: Calculate the Gowers norm of the flow pattern.
        let mut complex_sequence = values_to_complex_sequence_fft(&flow_sequence);
        let gowers_norm = calculate_u2_norm_fft(&mut complex_sequence);
        println!("  - Gowers Norm of Flow: {:.4}", gowers_norm);

        // 4. DECIDE: Compare to the target and determine policy adjustment.
        let error = gowers_norm - TARGET_GOWERS_NORM;
        println!("  - Norm Error: {:.4}", error);

        if error > 0.0 {
            // Flow is too structured; penalize the busiest path.
            println!("  - Policy: Flow is too structured. Increasing cost on busiest edge.");
            
            // Find the edge with the highest flow to penalize it.
            let mut busiest_edge_from = source;
            let mut busiest_edge_to = source;
            let mut max_flow_on_edge = 0;

            for (from_node, edges) in &graph.adj {
                for edge in edges {
                    if edge.flow > max_flow_on_edge {
                        max_flow_on_edge = edge.flow;
                        busiest_edge_from = *from_node;
                        busiest_edge_to = edge.to;
                    }
                }
            }
            
            // 5. ACT: Apply the new policy by increasing the cost.
            if let Some(edge) = graph.adj.get_mut(&busiest_edge_from).unwrap()
                .iter_mut().find(|e| e.to == busiest_edge_to) {
                edge.cost += COST_ADJUSTMENT_FACTOR;
                println!("  - Action: Cost of edge {:?} -> {:?} is now {}", busiest_edge_from, busiest_edge_to, edge.cost);
            }
        } else {
            println!("  - Policy: Flow complexity is at or below target. No cost changes.");
        }
        
        // Reset flow for the next iteration's calculation.
        for edges in graph.adj.values_mut() {
            for edge in edges {
                edge.flow = 0;
            }
        }
    }
    println!("\n--- Simulation Complete ---");
}

/// Converts the graph's flow values into a canonical, sorted sequence for analysis.
fn flow_to_sequence(graph: &Graph) -> Vec<f64> {
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

fn values_to_complex_sequence_fft(values: &[f64]) -> Vec<FftComplex<f64>> {
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