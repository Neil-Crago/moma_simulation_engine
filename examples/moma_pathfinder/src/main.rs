//! # A* Maze Solver Example

// This program demonstrates the use of the `pathfinder` module to:
// 1. Generate a random maze.
// 2. Solve the maze using the A* algorithm.
// 3. Draw the maze and the solved path to a JPEG image.

use image::{ImageBuffer, Rgb};
use moma_simulation_engine::grid::{Cell, Grid, Point};
use moma_simulation_engine::maze;
use moma_simulation_engine::pathfinding;
use std::fs::File;
use std::io::BufWriter;

fn main() -> std::io::Result<()> {
    println!("--- A* Maze Solver ---");

    // --- Configuration ---
    let width = 401;
    let height = 201;
    let start = Point::new(0, 1);
    let goal = Point::new(width - 1, height - 2);
    let scaling_factor = 4;

    // --- Maze Generation ---
    println!("Generating a {}x{} maze...", width, height);
    let mut grid = maze::generate_maze(width, height);
    println!("Maze generated.");

    // --- Pathfinding ---
    println!("Solving maze with A*...");
    if let Some(path) = pathfinding::a_star(&grid, start, goal) {
        println!("Path found with {} steps.", path.len());

        // Mark the path on the grid for drawing.
        for point in path {
            grid[point] = Cell::Path;
        }

        // --- Image Generation ---
        println!("Drawing maze to image...");
        draw_grid_to_jpeg(&grid, scaling_factor, "solved_maze.jpg")?;
        println!("Saved image to solved_maze.jpg");
    } else {
        println!("No path could be found from {:?} to {:?}.", start, goal);
    }

    Ok(())
}

/// Draws a grid to a JPEG file with a given scaling factor.
fn draw_grid_to_jpeg(grid: &Grid, scale: u32, filename: &str) -> std::io::Result<()> {
    let white = Rgb([255u8, 255u8, 255u8]);
    let black = Rgb([0u8, 0u8, 0u8]);
    let path_color = Rgb([89u8, 131u8, 152u8]); // A nice slate blue

    let img_width = grid.width() as u32 * scale;
    let img_height = grid.height() as u32 * scale;

    let img = ImageBuffer::from_fn(img_width, img_height, |x, y| {
        let grid_x = (x / scale) as usize;
        let grid_y = (y / scale) as usize;
        let point = Point::new(grid_x, grid_y);

        match grid[point] {
            Cell::Free => white,
            Cell::Blocked => black,
            Cell::Path => path_color,
        }
    });

    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);
    img.write_to(&mut writer, image::ImageFormat::Jpeg)
        .expect("Failed to write image");

    Ok(())
}
