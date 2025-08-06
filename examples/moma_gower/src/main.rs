//! Moma Pathfinder + Gower norm project
//! This project demonstrates the use of the `moma_simulation_engine` crate to solve a maze using the A* algorithm and to compute the Gower norm of the resulting path.
//! It generates a random maze, finds a path from the start to the goal, and then computes the Gower norm of the path.
//!
//! This program demonstrates the use of the `pathfinder` crate to:
//! 1. Generate a random maze.
//! 2. Solve the maze using the A* algorithm.
//! 3. use a Gower norm to evaluate the path.

use moma::core::{MomaRing, OriginStrategy};
use moma::strategy;
use moma_simulation_engine::automaton::Moma2dAutomaton;
use moma_simulation_engine::grid::Point;
use moma_simulation_engine::pathfinding::{Node, manhattan_distance};
use pixels::{Error, Pixels, SurfaceTexture};
use rustfft::{FftPlanner, num_complex::Complex as FftComplex};
//use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use winit::dpi::LogicalSize;
// 'WindowEvent' is no longer needed directly, so it's removed.
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

fn calculate_u2_norm_fft(sequence: &mut Vec<FftComplex<f64>>) -> f64 {
    let n = sequence.len();
    if n == 0 {
        return 0.0;
    }

    // 1. Setup the FFT planner
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);

    // 2. Perform the FFT on the sequence (in-place)
    fft.process(sequence);

    // 3. Calculate the sum of the 4th powers of the magnitudes
    let sum_of_magnitudes_pow4: f64 = sequence
        .iter()
        .map(|c| c.norm_sqr().powi(2)) // norm_sqr() is |c|^2. So this is (|c|^2)^2 = |c|^4
        .sum();

    // 4. Normalize the result and take the 4th root
    let norm = (sum_of_magnitudes_pow4 / (n as f64).powi(4)).powf(1.0 / 4.0);
    norm
}

fn a_star_moma_cost(
    automaton: &Moma2dAutomaton<impl OriginStrategy>,
    cost_ring: &MomaRing<impl OriginStrategy>,
    start: Point,
    goal: Point,
    structure_penalty_weight: f64,
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

    //cost = g(n) + 1.5 * h(n)
    //let cost = |g: u32, h: u32| g + (1.5 * h as f64) as u32;

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

            // Calculate a penalty based on the turning angle.
            let mut structure_penalty = 0.0;
            if let Some(&prev_point) = came_from.get(&current.point) {
                // We have a previous point, so we can calculate the turn.
                let dx1 = current.point.x as i32 - prev_point.x as i32;
                let dy1 = current.point.y as i32 - prev_point.y as i32;

                let dx2 = next_point.x as i32 - current.point.x as i32;
                let dy2 = next_point.y as i32 - current.point.y as i32;

                // If the vectors are the same (i.e., we're going straight), apply a penalty.
                if dx1 == dx2 && dy1 == dy2 {
                    structure_penalty = structure_penalty_weight; // High penalty for continuing straight
                }
            }

            let new_cost = cost_so_far[&current.point] + move_cost + (structure_penalty as u64);

            // Check if the next point is valid in the cost map
            // and if the new cost is lower than the existing cost.
            // If the next point is not in the cost map, it will return None.
            // If the next point is in the cost map, it will return Some(value).
            // This is a more idiomatic way to handle the cost check.
            // The `contains_key` method checks if the key exists in the map.
            // The `cost_so_far[&next_point]` will panic if the key does
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

/// Runs the dynamic pathfinding example using MOMA and A* algorithm with Gower norm.
/// This function sets up the event loop, initializes the automaton, and handles user input for dynamic pathfinding.
/// It draws the automaton state and the calculated path to the pixel buffer, allowing for real
/// time visualization of the pathfinding process.
fn dynamic_pathfinding() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("MOMA Dynamic A* + Gower Pathfinding")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let modulus = 16;
    let mut automaton =
        Moma2dAutomaton::new(WIDTH as usize, HEIGHT as usize, modulus, strategy::PrimeGap);
    let cost_ring = MomaRing::new(modulus, strategy::CompositeMass);
    let start = Point {
        x: 10,
        y: HEIGHT as usize / 2,
    };
    let goal = Point {
        x: WIDTH as usize - 10,
        y: HEIGHT as usize / 2,
    };
    let mut path: Option<Vec<Point>> = None;

    let target_norm = 0.25; // 0.58 works better!

    // If we are making progress (error is decreasing), it's okay to increase the penalty.
    // But if the error is stuck or increasing, we should slowly decay the penalty
    // to see if we can get the same result with less effort.
    // This requires storing the previous frame's error.
    //let mut last_error = 0.0;

    let mut structure_penalty_weight: f64 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current state
        if let Event::RedrawRequested(_) = event {
            draw(pixels.frame_mut(), &automaton, &path);
            if let Err(err) = pixels.render() {
                eprintln!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            // The `.quit()` method is deprecated. Use `.close_requested()` instead.
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            automaton.step();
            // Pass the CURRENT policy to the pathfinder
            path = a_star_moma_cost(
                &automaton,
                &cost_ring,
                start,
                goal,
                structure_penalty_weight,
            );

            let maze_path: Vec<(i32, i32)> = path
                .as_ref()
                .map(|p| p.iter().map(|pt| (pt.x as i32, pt.y as i32)).collect())
                .unwrap_or_default();

            let mut p_to_complexfft = path_to_complex_sequence_fft(&maze_path);
            let u2_norm_fft = calculate_u2_norm_fft(&mut p_to_complexfft);
            if u2_norm_fft < 0.0001 {
                println!("Path Norm is too small, resetting penalty weight.");
                structure_penalty_weight = 0.0; // Reset penalty weight if norm is too small
            }


//let target_norm = 0.25;
let error = u2_norm_fft - target_norm;

// --- New, Smoother PI Controller Logic ---

// 1. A small, proportional gain to nudge the penalty in the right direction.
//    Note: This gain is much smaller than my previous '50.0'.
let proportional_gain = 5.0; 

// 2. A decay rate to represent the "cost of effort".
let decay_rate = 0.01; // Equivalent to multiplying by 0.99

// 3. Calculate the adjustment based on the current error.
let adjustment = error * proportional_gain;

// 4. Apply the new logic: decay the old weight and add the new adjustment.
structure_penalty_weight = (structure_penalty_weight * (1.0 - decay_rate)) + adjustment;

// Ensure the penalty never goes below zero.
structure_penalty_weight = structure_penalty_weight.max(0.0);

//last_error = error;

println!(
                "Path Norm: {:.3}, Target: {:.3}, Penalty Weight: {:.3}",
                u2_norm_fft, target_norm, structure_penalty_weight
            );
            window.request_redraw();
        }
    });
}

/// Draws the automaton grid and the calculated path to the pixel buffer.
fn draw(
    frame: &mut [u8],
    automaton: &Moma2dAutomaton<impl OriginStrategy>,
    path: &Option<Vec<Point>>,
) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let cell_state = automaton.state[i];
        let terrain_color = state_to_color(cell_state);
        pixel.copy_from_slice(&terrain_color);
    }

    if let Some(path_points) = path {
        for point in path_points {
            let i = point.y * automaton.width + point.x;
            if let Some(pixel) = frame.chunks_exact_mut(4).nth(i) {
                pixel.copy_from_slice(&[255, 255, 0, 255]); // Bright Yellow
            }
        }
    }
}

/// Maps a cell state to a color for visualization (cool blues to warm reds).
fn state_to_color(state: u64) -> [u8; 4] {
    let ratio = state as f32 / 16.0; // Assuming modulus 16
    let r = (200.0 * ratio) as u8 + 55;
    let g = 55;
    let b = (200.0 * (1.0 - ratio)) as u8 + 55;
    [r, g, b, 255]
}

fn path_to_complex_sequence_fft(path: &Vec<(i32, i32)>) -> Vec<FftComplex<f64>> {
    let mut complex_sequence: Vec<FftComplex<f64>> = Vec::new();
    if path.len() < 2 {
        return complex_sequence;
    }

    for p in 1..path.len() {
        let dx = path[p].0 - path[p - 1].0;
        let dy = path[p].1 - path[p - 1].1;
        let angle = (dy as f64).atan2(dx as f64);
        complex_sequence.push(FftComplex::new(angle.cos(), angle.sin()));
    }
    complex_sequence
}

fn main() {
    dynamic_pathfinding().unwrap_or_else(|e| {
        eprintln!("Error running dynamic pathfinding: {}", e);
    });
    println!("\n--- End of Dynamic A* + Gower Pathfinding Example ---\n");
    /*
    let path = run_pathfinding();
    if let Err(e) = path.1 {
        eprintln!("Error running pathfinding: {}", e);
        return;
    }

    println!("\ntest with path to complex data\n");
    let straight_line = vec![(0,0), (1,0), (2,0), (3,0), (4,0), (5,0)];
    let staircase = vec![(0,0), (1,0), (1,1), (2,1), (2,2), (3,2)];
    let maze_path: Vec<(i32, i32)> = path.0.iter().map(|p| (p.x as i32, p.y as i32)).collect();

    let mut p1 = path_to_complex_sequence_fft(&straight_line);
    let mut p2 = path_to_complex_sequence_fft(&staircase);
    let mut p3 = path_to_complex_sequence_fft(&maze_path);

    let c1 = calculate_u2_norm_fft(&mut p1);
    let c2 = calculate_u2_norm_fft(&mut p2);
    let c3 = calculate_u2_norm_fft(&mut p3);

    println!("straight_line = {c1:>3.9}");
    println!("staircase = {c2:>3.9}");
    println!("maze_path = {c3:>3.9}");
    */
}
