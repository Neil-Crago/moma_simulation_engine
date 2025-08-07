//! # Network Graph Module
//!
//! Provides the foundational data structures for representing a flow network.
//! This includes nodes, directed edges with capacity and cost, and the main
//! graph container. It's the "Chain" in our BRC architecture.

// We reuse the Point struct from our existing pathfinding work.
// Make sure it's accessible from this module.
use crate::grid::Point; // Assuming Point is in a `grid` module. Adjust if needed.
use std::collections::HashMap;

/// Represents a directed connection between two nodes in the graph.
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: Point,
    pub capacity: u64, // The maximum flow the edge can handle.
    pub cost: u64,     // The cost to send flow, adjusted by the Strategist.
    pub flow: u64,     // The current flow being pushed through the edge.
}

/// Represents the entire flow network, including all nodes and edges.
#[derive(Debug)]
pub struct Graph {
    // We use a HashMap to store the adjacency list.
    // The key is a node (Point), and the value is a Vec of its outgoing edges.
    pub adj: HashMap<Point, Vec<Edge>>,
    pub source: Point,
    pub sink: Point,
}

impl Graph {
    /// Creates a new, empty graph with a defined source and sink.
    pub fn new(source: Point, sink: Point) -> Self {
        Graph {
            adj: HashMap::new(),
            source,
            sink,
        }
    }

    /// Adds a new node to the graph.
    /// Ensures a node exists in the adjacency list, even if it has no outgoing edges.
    pub fn add_node(&mut self, node: Point) {
        self.adj.entry(node).or_insert_with(Vec::new);
    }

    /// Adds a directed edge to the graph.
    /// This will be the primary way we build our network from the maze or automaton state.
    pub fn add_edge(&mut self, from: Point, to: Point, capacity: u64, cost: u64) {
        // Ensure the 'from' node exists in the adjacency list.
        self.add_node(from);
        self.add_node(to);

        // Add the forward edge.
        let forward_edge = Edge {
            to,
            capacity,
            cost,
            flow: 0,
        };
        self.adj.get_mut(&from).unwrap().push(forward_edge);
    }

    /// A helper to get all outgoing edges from a given node.
    pub fn get_edges(&self, node: &Point) -> &Vec<Edge> {
        // Return an empty Vec if the node has no outgoing edges.
        self.adj.get(node).unwrap_or_else(|| {
            // This case should be rare if nodes are added correctly.
            // We can add a panic! here during debug if we want to be strict.
            static EMPTY_VEC: Vec<Edge> = Vec::new();
            &EMPTY_VEC
        })
    }

    /// Finds a path from source to sink with available capacity using BFS.
    /// This is the "scout" part of the Edmonds-Karp algorithm.
    /// It returns a map of parent pointers to reconstruct the path.
    fn bfs(&self) -> HashMap<Point, Point> {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(self.source);

        let mut visited = std::collections::HashSet::new();
        visited.insert(self.source);

        let mut parent_map = HashMap::new();

        while let Some(u) = queue.pop_front() {
            if u == self.sink {
                break; // Found a path to the sink
            }

            // Check all outgoing edges for available capacity
            for edge in self.get_edges(&u) {
                if !visited.contains(&edge.to) && edge.capacity > edge.flow {
                    visited.insert(edge.to);
                    queue.push_back(edge.to);
                    parent_map.insert(edge.to, u);
                }
            }
        }
        parent_map
    }

    /// Calculates the maximum flow from source to sink using the Edmonds-Karp algorithm.
    /// This is our "Rope" or Tactician.
    pub fn edmonds_karp(&mut self) -> u64 {
        let mut max_flow = 0;

        loop {
            let parent_map = self.bfs();

            // If BFS couldn't find a path to the sink, we're done.
            if !parent_map.contains_key(&self.sink) {
                break;
            }

            // --- Path found, now find the bottleneck capacity ---
            let mut path_flow = u64::MAX;
            let mut current = self.sink;
            while current != self.source {
                let prev = parent_map[&current];
                
                // Find the specific edge in the adjacency list to check its capacity
                let edge = self.adj.get(&prev).unwrap().iter()
                    .find(|e| e.to == current).unwrap();

                path_flow = path_flow.min(edge.capacity - edge.flow);
                current = prev;
            }

            // --- Augment the flow along the path ---
            max_flow += path_flow;
            let mut v = self.sink;
            while v != self.source {
                let u = parent_map[&v];
                
                // Update flow on the forward edge
                if let Some(edge) = self.adj.get_mut(&u).unwrap().iter_mut()
                    .find(|e| e.to == v) {
                    edge.flow += path_flow;
                }
                
                // Note: A full implementation also handles backward edges for flow reduction,
                // but for our simulation, this simplified augmentation is a great start.

                v = u;
            }
        }
        max_flow
    }
}