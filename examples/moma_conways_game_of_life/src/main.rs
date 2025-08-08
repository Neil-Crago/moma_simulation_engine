//! MOMA Conway's Game of Life Example
//
// A visual demonstration of Conway's Game of Life where the initial state
// is generated using a MOMA signature sequence.

use moma::core::MomaRing;
use moma::strategy;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
// This trait is now guaranteed to be the correct version.
use rand::SeedableRng;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

/// Represents the state of the Game of Life simulation.
struct World {
    cells: Vec<u8>,
}

impl World {
    /// Creates a new World with a MOMA-seeded initial state of gliders.
    fn new() -> Self {
        let mut cells = vec![0; (WIDTH * HEIGHT) as usize];
        let mut rng = ChaCha8Rng::from_seed([42; 32]);

        // MOMA rings to generate (x, y) coordinates for the gliders.
        let ring_x = MomaRing::new(WIDTH as u64, strategy::CompositeMass);
        let ring_y = MomaRing::new(HEIGHT as u64, strategy::PrimeGap);

        // A glider pattern. It's a 3x3 shape.
        let glider: [(isize, isize); 5] = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];

        let mut p = 3;
        // Place 50 gliders on the grid.
        for _ in 0..50 {
            // 1. Get a random top-left position for the glider using MOMA.
            let sig_x = ring_x.signature(p) as isize;
            p = moma::primes::next_prime(p + 1);
            let sig_y = ring_y.signature(p) as isize;
            p = moma::primes::next_prime(p + 1);

            // 2. Get a random orientation (0-3 for normal, flipped, etc.).
            let orientation = rng.gen_range(0..4);

            // 3. "Stamp" the glider pattern onto the grid cells.
            for (mut dx, mut dy) in glider {
                // Apply a random rotation/flip to the glider pattern
                match orientation {
                    1 => dx = -dx, // Flipped horizontally
                    2 => dy = -dy, // Flipped vertically
                    3 => { (dx, dy) = (dy, dx) } // Rotated
                    _ => {}
                }

                let x = (sig_x + dx + WIDTH as isize) as u32 % WIDTH;
                let y = (sig_y + dy + HEIGHT as isize) as u32 % HEIGHT;
                let index = (y * WIDTH + x) as usize;
                cells[index] = 1;
            }
        }

        Self { cells }
    }

    /// Updates the simulation by one step.
    fn update(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..HEIGHT as i32 {
            for x in 0..WIDTH as i32 {
                let idx = (y * WIDTH as i32 + x) as usize;
                let neighbors = self.count_neighbors(x, y);

                let next_cell = match (self.cells[idx], neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours dies.
                    (1, n) if n < 2 => 0,
                    // Rule 2: Any live cell with two or three live neighbours lives on.
                    (1, 2) | (1, 3) => 1,
                    // Rule 3: Any live cell with more than three live neighbours dies.
                    (1, n) if n > 3 => 0,
                    // Rule 4: Any dead cell with exactly three live neighbours becomes a live cell.
                    (0, 3) => 1,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };
                next[idx] = next_cell;
            }
        }
        self.cells = next;
    }

    /// Counts the live neighbors of a given cell.
    fn count_neighbors(&self, x: i32, y: i32) -> u8 {
        let mut count = 0;
        for dy in [-1, 0, 1] {
            for dx in [-1, 0, 1] {
                if dx == 0 && dy == 0 { continue; }

                let nx = (x + dx + WIDTH as i32) as u32 % WIDTH;
                let ny = (y + dy + HEIGHT as i32) as u32 % HEIGHT;
                let idx = (ny * WIDTH + nx) as usize;
                count += self.cells[idx];
            }
        }
        count
    }

    /// Draws the world to the pixel buffer.
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = if self.cells[i] == 1 {
                [0x5e, 0x48, 0xe8, 0xff] // A nice purple
            } else {
                [0x48, 0xb2, 0xe8, 0xff] // A soft blue
            };
            pixel.copy_from_slice(&color);
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("MOMA Conway's Game of Life")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new();

event_loop.run(move |event, _, control_flow| {
    // Handle input events first.
    if input.update(&event) {
        // Close events
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
    }

    // This is the new logic for continuous updates and drawing.
    match event {
        Event::MainEventsCleared => {
            // Update the world state.
            world.update();
            // Request a redraw.
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            // Draw the world to the buffer.
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                eprintln!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
            }
        }
        _ => (),
    }
});
}