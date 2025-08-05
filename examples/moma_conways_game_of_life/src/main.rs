//! # MOMA Pixels Automaton Example
//!
//! This example demonstrates how to visualize the MOMA-powered cellular automaton
//! in real-time using the `pixels` crate for GPU-accelerated rendering.
//!
//! It creates a window, initializes a 2D cellular automaton, and then runs a
//! simulation loop where each generation is drawn to the screen as a colorful grid.

use moma::strategy;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use moma_simulation_engine::automaton::CellularAutomaton; // Assuming this is now 2D
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// --- 2D Cellular Automaton (Adapted for this example) ---
// NOTE: For this to work, you would adapt your `simulation_engine`'s
// `CellularAutomaton` to be 2D. For now, we'll include a 2D version here.

/// Represents a 2D Cellular Automaton whose rules are governed by MOMA.
pub struct Moma2dAutomaton<S: moma::core::OriginStrategy> {
    state: Vec<u64>,
    width: usize,
    height: usize,
    ring: moma::core::MomaRing<S>,
}

impl<S: moma::core::OriginStrategy + Clone> Moma2dAutomaton<S> {
    /// Creates a new 2D Automaton with a random initial state.
    pub fn new(width: usize, height: usize, modulus: u64, strategy: S) -> Self {
        let mut rng = rand::rng();
        let size = width * height;
        let state = (0..size).map(|_| rng.random_range(0..modulus)).collect();

        Self {
            state,
            width,
            height,
            ring: moma::core::MomaRing::new(modulus, strategy),
        }
    }

    /// Advances the simulation by one time step.
    pub fn step(&mut self) {
        let mut next_state = self.state.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                // Get Moore neighborhood (8 neighbors), wrapping around the edges.
                let mut neighbor_sum = 0;
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as isize + dx + self.width as isize) as usize % self.width;
                        let ny = (y as isize + dy + self.height as isize) as usize % self.height;
                        neighbor_sum += self.state[ny * self.width + nx];
                    }
                }

                let current_index = y * self.width + x;
                let center = self.state[current_index];

                // The MOMA Update Rule:
                let new_value = self.ring.residue(center, neighbor_sum);
                next_state[current_index] = new_value;
            }
        }
        self.state = next_state;
    }

    /// Provides a slice to the current state for rendering.
    pub fn state(&self) -> &[u64] {
        &self.state
    }
}


// --- Main Application ---

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("MOMA Cellular Automaton")
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

    // --- Automaton Setup ---
    let modulus = 16;
    let strategy = strategy::PrimeGap; // Try changing this to CompositeMass!
    let mut automaton = Moma2dAutomaton::new(WIDTH as usize, HEIGHT as usize, modulus, strategy);

    // --- Color Palette Setup ---
    // Generate a vibrant, random color palette for the cell states.
    let mut rng = ChaCha8Rng::seed_from_u64(1337);
    let palette: Vec<[u8; 4]> = (0..modulus)
        .map(|_| {
            let r = rng.gen_range(50..=255);
            let g = rng.gen_range(50..=255);
            let b = rng.gen_range(50..=255);
            [r, g, b, 0xff]
        })
        .collect();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current automaton state
        if let Event::RedrawRequested(_) = event {
            draw(pixels.frame_mut(), &automaton, &palette);
            if let Err(err) = pixels.render() {
                eprintln!("pixels.render() failed: {}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("pixels.resize_surface() failed: {}", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            automaton.step();
            window.request_redraw();
        }
    });
}

/// Draws the automaton's state to the pixel buffer.
fn draw(frame: &mut [u8], automaton: &Moma2dAutomaton<impl moma::core::OriginStrategy>, palette: &[[u8; 4]]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let cell_state = automaton.state()[i] as usize;
        let color = palette[cell_state];
        pixel.copy_from_slice(&color);
    }
}
