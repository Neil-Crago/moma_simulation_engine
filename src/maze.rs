//! # Maze Generation Module
//!
//! Provides functions for generating random mazes using a randomized
//! depth-first search algorithm.

use crate::grid::{Cell, Grid, Point};
use rand::seq::SliceRandom;

/// Generates a random maze of a given size.
///
/// The maze is guaranteed to have a path from `(0, 1)` to `(width - 1, height - 2)`.
/// The algorithm works by carving paths from a solid grid of `Blocked` cells.
///
/// # Arguments
/// * `width` - The width of the maze. Must be an odd number.
/// * `height` - The height of the maze. Must be an odd number.
pub fn generate_maze(width: usize, height: usize) -> Grid {
    assert!(width % 2 != 0 && height % 2 != 0, "Width and height must be odd.");

    let mut grid = Grid::new(width, height, Cell::Blocked);
    let mut stack: Vec<Point> = Vec::new();
    let mut rng = rand::rng();

    // Start carving from the center of the grid.
    let start_point = Point::new(1, 1);
    grid[start_point] = Cell::Free;
    stack.push(start_point);

    while let Some(current) = stack.last().copied() {
        let mut directions = [(-2, 0), (2, 0), (0, -2), (0, 2)];
        directions.shuffle(&mut rng);

        let mut moved = false;
        for (dx, dy) in directions {
            let nx = current.x as isize + dx;
            let ny = current.y as isize + dy;

            if nx > 0 && nx < width as isize - 1 && ny > 0 && ny < height as isize - 1 {
                let next_point = Point::new(nx as usize, ny as usize);
                if grid[next_point] == Cell::Blocked {
                    // Carve path to the new cell
                    grid[next_point] = Cell::Free;
                    // Carve path in the wall between cells
                    let wall_point = Point::new((current.x as isize + dx / 2) as usize, (current.y as isize + dy / 2) as usize);
                    grid[wall_point] = Cell::Free;

                    stack.push(next_point);
                    moved = true;
                    break;
                }
            }
        }

        if !moved {
            stack.pop(); // Backtrack
        }
    }

    // Create an entrance and an exit.
    grid[Point::new(0, 1)] = Cell::Free;
    grid[Point::new(width - 1, height - 2)] = Cell::Free;

    grid
}
