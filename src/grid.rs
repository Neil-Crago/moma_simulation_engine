//! # Grid Module
//!
//! Provides the fundamental data structures for working with a 2D grid,
//! including `Point`, `Cell` state, and the `Grid` itself.

use std::ops::{Index, IndexMut};

/// Represents a 2D coordinate on the grid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Represents the state of a single cell within the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    /// An impassable wall.
    Blocked,
    /// An open space that can be traversed.
    Free,
    /// A cell that is part of the calculated path.
    Path,
}

/// Represents a 2D grid of cells.
#[derive(Debug, Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Creates a new grid of a given size, initialized with a specific cell type.
    pub fn new(width: usize, height: usize, initial_cell: Cell) -> Self {
        Self {
            width,
            height,
            cells: vec![initial_cell; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns an iterator over the valid neighbors of a given point.
    /// A neighbor is valid if it is within the grid bounds and is not blocked.
    pub fn neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        [(-1, 0), (1, 0), (0, -1), (0, 1)] // Left, Right, Up, Down
            .iter()
            .filter_map(move |&(dx, dy)| {
                let nx = point.x as isize + dx;
                let ny = point.y as isize + dy;

                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let neighbor_point = Point::new(nx as usize, ny as usize);
                    if self[neighbor_point] != Cell::Blocked {
                        return Some(neighbor_point);
                    }
                }
                None
            })
    }
}

// Allow accessing grid cells using `grid[point]` syntax.
impl Index<Point> for Grid {
    type Output = Cell;
    fn index(&self, point: Point) -> &Self::Output {
        &self.cells[point.y * self.width + point.x]
    }
}

// Allow mutating grid cells using `grid[point] = Cell::Path` syntax.
impl IndexMut<Point> for Grid {
    fn index_mut(&mut self, point: Point) -> &mut Self::Output {
        &mut self.cells[point.y * self.width + point.x]
    }
}
