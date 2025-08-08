//! # Network Graph Module
//
// Provides the foundational data structures for representing a flow network.
// This includes nodes, directed edges with capacity and cost, and the main
// graph container. It's the "Chain" in our BRC architecture.

// We reuse the Point struct from our existing pathfinding work.
// Make sure it's accessible from this module.
use crate::grid::Point; // Assuming Point is in a `grid` module. Adjust if needed.
use std::collections::HashMap;
use ordered_float::OrderedFloat;
use std::collections::BinaryHeap;

/// Represents a directed connection between two nodes in the graph.
#[derive(Debug, Clone)]
pub struct Edge {
    pub to: Point,
    pub capacity: u64,
    pub cost: f64,     // Changed from u64 to i64
    pub flow: u64,
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
   pub fn add_edge(&mut self, from: Point, to: Point, capacity: u64, cost: f64) { // Update signature
        self.add_node(from);
        self.add_node(to);

        self.adj.get_mut(&from).unwrap().push(Edge {
            to,
            capacity,
            cost,
            flow: 0,
        });
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

    /// Finds the cheapest path from source to sink using Dijkstra's algorithm.
    /// This version is cost-aware and replaces the simple BFS.
    /// It returns a map of parent pointers to reconstruct the path.

    fn find_cheapest_path_dijkstra(&self) -> (HashMap<Point, Point>, bool) {
        let mut distances: HashMap<Point, f64> = HashMap::new();
        let mut parent_map = HashMap::new();
        let mut pq = BinaryHeap::new();

        distances.insert(self.source, 0.0);
        // We use OrderedFloat to allow f64 in the max-heap.
        // We still negate to make it a min-heap.
        pq.push((OrderedFloat(-0.0), self.source));

        while let Some((cost, u)) = pq.pop() {
            let cost = -cost.into_inner(); // unwrap the OrderedFloat

            if cost > *distances.get(&u).unwrap_or(&f64::MAX) {
                continue;
            }
            if u == self.sink {
                return (parent_map, true);
            }

            for edge in self.get_edges(&u) {
                if edge.capacity > edge.flow {
                    let new_dist = cost + edge.cost;
                    if new_dist < *distances.get(&edge.to).unwrap_or(&f64::MAX) {
                        distances.insert(edge.to, new_dist);
                        pq.push((OrderedFloat(-new_dist), edge.to));
                        parent_map.insert(edge.to, u);
                    }
                }
            }
        }
        (parent_map, distances.contains_key(&self.sink))
    }
    
    /// Calculates the maximum flow, now using a cost-aware pathfinding method.
    pub fn edmonds_karp(&mut self) -> u64 {
        let mut max_flow = 0;
        loop {
            // Use the new Dijkstra-based pathfinder
            let (parent_map, sink_found) = self.find_cheapest_path_dijkstra();

            if !sink_found {
                break; // No more paths to the sink
            }

            // --- Path found, find bottleneck capacity ---
            let mut path_flow = u64::MAX;
            let mut current = self.sink;
            while current != self.source {
                let prev = parent_map[&current];
                let edge = self.adj.get(&prev).unwrap().iter()
                    .find(|e| e.to == current).unwrap();
                path_flow = path_flow.min(edge.capacity - edge.flow);
                current = prev;
            }

            // --- Augment flow ---
            max_flow += path_flow;
            let mut v = self.sink;
            while v != self.source {
                let u = parent_map[&v];
                if let Some(edge) = self.adj.get_mut(&u).unwrap().iter_mut()
                    .find(|e| e.to == v) {
                    edge.flow += path_flow;
                }
                v = u;
            }
        }
        max_flow
    }


    /// Finds the single cheapest path and routes flow down it.
    /// This replaces edmonds_karp to act as a policy-driven Tactician.

    /// Finds the single cheapest path and routes flow, returning the flow and the path itself.
    pub fn route_cheapest_path(&mut self) -> (u64, Option<Vec<Point>>) {
        let (parent_map, sink_found) = self.find_cheapest_path_dijkstra();

        if !sink_found {
            return (0, None);
        }

        // --- Reconstruct the path ---
        let mut path = vec![self.sink];
        let mut current = self.sink;
        while current != self.source {
            current = parent_map[&current];
            path.push(current);
        }
        path.reverse();
        let path_clone = path.clone(); // Clone it to return later

        // --- Calculate bottleneck and push flow ---
        let mut path_flow = u64::MAX;
        for i in 0..path.len() - 1 {
            let u = path[i];
            let v = path[i+1];
            let edge = self.adj.get(&u).unwrap().iter().find(|e| e.to == v).unwrap();
            path_flow = path_flow.min(edge.capacity - edge.flow);
        }

        for i in 0..path.len() - 1 {
            let u = path[i];
            let v = path[i+1];
            if let Some(edge) = self.adj.get_mut(&u).unwrap().iter_mut().find(|e| e.to == v) {
                edge.flow += path_flow;
            }
        }
        
        (path_flow, Some(path_clone))
    }
}