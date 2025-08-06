//! # A* Pathfinding Module
//!
//! Provides a generic implementation of the A* search algorithm.

use crate::grid::{Grid, Point};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

type Cost = u32;

/// Represents a node in the A* search space.
#[derive(Debug, Eq, PartialEq)]
pub struct Node {
    pub point: Point,
    /// The cost from the start node to this node (g-cost).
    pub cost: Cost,
    /// The estimated cost from this node to the goal (h-cost, the heuristic).
    pub heuristic: Cost,
}

// The priority queue (BinaryHeap) needs `Ord`. We want to pop the node with the
// lowest total cost (cost + heuristic), so we reverse the comparison.
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The Manhattan distance heuristic for a grid.
pub fn manhattan_distance(a: Point, b: Point) -> Cost {
    ((a.x as i32 - b.x as i32).abs() + (a.y as i32 - b.y as i32).abs()) as Cost
}

/// Finds the shortest path from a start to a goal point in a grid using the A* algorithm.
///
/// # Arguments
/// * `grid` - The grid to search in.
/// * `start` - The starting point of the path.
/// * `goal` - The target point of the path.
///
/// # Returns
/// `Some(Vec<Point>)` containing the path from start to goal if one is found,
/// otherwise `None`.
pub fn a_star(grid: &Grid, start: Point, goal: Point) -> Option<Vec<Point>> {
    let mut frontier = BinaryHeap::new();
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut cost_so_far: HashMap<Point, Cost> = HashMap::new();

    cost_so_far.insert(start, 0);
    frontier.push(Node {
        point: start,
        cost: 0,
        heuristic: manhattan_distance(start, goal),
    });

    while let Some(current) = frontier.pop() {
        if current.point == goal {
            // We found the goal, reconstruct the path.
            let mut path = vec![goal];
            let mut curr = goal;
            while curr != start {
                curr = came_from[&curr];
                path.push(curr);
            }
            path.reverse();
            return Some(path);
        }

        for next_point in grid.neighbors(current.point) {
            let new_cost = cost_so_far[&current.point] + 1; // Cost of moving is always 1.

            if !cost_so_far.contains_key(&next_point) || new_cost < cost_so_far[&next_point] {
                cost_so_far.insert(next_point, new_cost);
                let priority = manhattan_distance(next_point, goal);
                frontier.push(Node {
                    point: next_point,
                    cost: new_cost,
                    heuristic: priority,
                });
                came_from.insert(next_point, current.point);
            }
        }
    }

    None // No path found
}
