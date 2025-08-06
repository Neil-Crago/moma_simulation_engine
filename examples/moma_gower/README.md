# MOMA-Gowers Pathfinder

[](https://opensource.org/licenses/MIT)

An experimental simulation in Rust demonstrating a self-regulating AI agent. This project uses an A\* pathfinding algorithm to navigate a dynamic, procedurally generated world. The agent's decision-making is governed by a **MOMA (Multi-Objective Meta-Observer Architecture)** feedback loop, which uses **Gowers Uniformity Norms** to analyze the geometric complexity of its own solution paths and adapt its strategy in real-time.

## Core Concepts

This project integrates three advanced concepts to create its unique behaviour:

### 1\. MOMA (Multi-Objective Meta-Observer Architecture)

The agent's intelligence is structured as a set of **nested rings**, inspired by the OODA loop (Observe-Orient-Decide-Act).

  * **The Inner "Tactician" Ring:** A fast loop responsible for executing a specific task—in this case, running the A\* algorithm to find a path from a start to a goal based on a given policy.
  * **The Outer "Strategist" Ring:** A slower, reflective loop that observes the results of the Inner Ring's actions. It analyzes the generated paths and "Decides" on a new policy or strategy, which it then passes down to the Tactician. This creates a system that learns from and adapts its own behaviour.

### 2\. Gowers Uniformity Norms

We use the Gowers $U^2$-norm as a mathematical "structure detector." It is a tool from additive combinatorics that allows us to distill the complex geometry of a path into a single number representing its regularity or predictability.

  * A **high norm (near 1.0)** indicates a very structured path (e.g., a straight line).
  * A **low norm (near 0.0)** indicates a chaotic, geometrically complex, or unpredictable path.

The calculation is performed efficiently using a **Fast Fourier Transform (FFT)** based method, allowing for real-time analysis of long paths.

### 3\. The Feedback Loop

The MOMA rings and the Gowers norm analysis are connected in a homeostatic feedback loop.

1.  The **Tactician** finds a path.
2.  The path is immediately analyzed to calculate its **Gowers norm**.
3.  The **Strategist** observes this norm and compares it to a `target_norm` (its goal).
4.  Based on the error, it calculates and adjusts a `structure_penalty_weight`.
5.  This penalty is fed back into the **Tactician's** A\* cost function for the *next* frame, making predictable moves more "expensive."

This loop allows the agent to dynamically adjust its pathfinding logic to actively pursue abstract goals like "be more unpredictable."

-----

## System Architecture

The data flows from the pathfinder to the analysis tools and back into the pathfinder's policy on the next frame.

```mermaid
graph TD
    subgraph Outer Ring (The Strategist)
        A[Observe Path & Norm] --> B[Orient: Compare norm to target];
        B --> C[Decide: Adjust Penalty Weight];
        C --> D[Act: Set New Policy for Inner Ring];
    end

    subgraph Inner Ring (The Tactician)
        E[Run A* with current policy] --> F[Generate Path];
    end
    
    D -.-> E;
    F --> G[path_to_complex_sequence];
    G --> H[calculate_u2_norm_fft];
    H --> A;
```

-----

## Getting Started

### Prerequisites

You need to have the Rust programming language and its package manager, Cargo, installed. You can get them from [rust-lang.org](https://www.rust-lang.org/).

### Installation & Building

1.  Clone this repository to your local machine.
2.  Navigate to the project directory in your terminal.
3.  Build the project. For the best performance, use a release build:
    ```bash
    cargo build --release
    ```

### Running the Simulation

Execute the compiled binary from the project root:

```bash
cargo run --release
```

  * A window will appear showing the dynamic environment.
  * The yellow line represents the path calculated by the agent on each frame.
  * The console will print the **Path Norm**, **Target Norm**, and **Penalty Weight** for each frame, allowing you to observe the feedback loop in action.
  * Press the **`Escape`** key to close the simulation.

-----

## Configuration & Tuning

You can experiment with the agent's strategic mind by tweaking the parameters in the `dynamic_pathfinding` function.

```rust
// In dynamic_pathfinding()

// The Outer Ring's goal. A lower value encourages more complex paths.
let target_norm = 0.25;

// ... inside the event loop ...

// --- The Strategist's "Decide" Logic ---

// Determines how aggressively the agent reacts to errors.
let proportional_gain = 5.0; 

// Represents the "cost of effort" and helps stabilize the system.
let decay_rate = 0.01; 

// The core control logic
let adjustment = error * proportional_gain;
structure_penalty_weight = (structure_penalty_weight * (1.0 - decay_rate)) + adjustment;
```

-----

## Future Work

This project serves as a foundation for more advanced explorations:

  * **Project 2: The Forensic Investigator:** Implement the Inverse Gowers Theorem to not just *detect* structure, but to *identify* its specific type (e.g., linear, quadratic).
  * **Higher-Order Norms:** Implement the $U^3$ and $U^4$ norms to detect more complex, non-linear patterns in agent behaviour.
  * **More Sophisticated Policies:** Evolve the Outer Ring's "Decide" logic to handle multiple competing objectives simultaneously (e.g., balance path complexity against path length and risk).