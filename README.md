# MOMA Simulation Engine

[![Crates.io](https://img.shields.io/crates/v/moma_simulation_engine.svg?style=flat-square)](https://crates.io/crates/moma_simulation_engine)
[![Docs.rs](https://img.shields.io/docsrs/moma_simulation_engine?style=flat-square)](https://docs.rs/moma_simulation_engine)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](https://opensource.org/licenses/MIT)

The companion to [`moma`](https://crates.io/crates/moma). moma analyses spirals and helices around a wandering axis; this crate **makes** such tracks with a known true axis, so moma's estimators can be scored against the truth instead of judged by eye.

Its only dependency is `moma` 1.0. Everything is deterministic from a seed (it carries its own small generator).

> **Status:** everything below is tested on synthetic data only. Passing here shows an estimator does what it claims when the model is right; it does not show it works on real tracks. For real-data checks see the moma README.

## What is in it

| Module | Purpose |
|---|---|
| `track` | `SpiralSpec` / `HelixSpec`: a rotation about a wandering axis. Axis motion: fixed, drift, random walk, wobble. Rotation: steady, irregular, reversing. Plus ellipticity, Gaussian noise and dropped samples. Returns points with the true axis and phase. |
| `evaluation` | `Estimator`: each axis method in `moma::spiral` behind one interface. `evaluate` scores them against the true axis; `consensus` merges them with `moma::consensus::consensus_axis_2d` and scores the result. |
| `automaton` | `ConsensusAutomaton`: a 1D or toroidal 2D cellular automaton whose rule is the circular mean on `Z_m` (`moma::consensus::ring_consensus`), with order and local-agreement measures. |
| `rng` | SplitMix64 with uniform, bounded-integer and Gaussian draws. |

```rust
use moma_simulation_engine::{Estimator, SpiralSpec, Wander, evaluate};

let track = SpiralSpec::default()
    .with_wander(Wander::Drift { vx: 0.02, vy: 0.0 })
    .with_noise(0.02)
    .simulate()?;
for r in evaluate(&track, &Estimator::suite(track.period)) {
    println!("{:10} {:?}", r.name, r.axis_error);
}
```

Invalid parameters (zero length, period of 2 samples or less, negative noise, ...) return `Err`, never panic.

## Results (synthetic)

`cargo run --release --example axis_bench`: RMS axis error in units of the orbit radius, 1500 samples, period 40, aspect 0.6, wobbling axis (amplitude 0.8, period 500). The first half period at each end is excluded.

| scenario | fixed | adaptive | auto | period | median | midrange | coverage | refined | consensus |
|---|---|---|---|---|---|---|---|---|---|
| steady, wobble | 0.022 | 0.022 | 0.022 | **0.008** | 0.070 | 0.057 | 0.062 | 0.120 | 0.022 |
| steady, walk | 0.078 | 0.081 | 0.078 | 0.077 | 0.104 | 0.093 | 0.094 | 0.136 | 0.078 |
| steady, wobble, noise 0.1 | 0.030 | 0.057 | 0.030 | **0.023** | 0.095 | 0.082 | 0.085 | 0.125 | 0.030 |
| steady, wobble, 15% dropout | 0.022 | 0.022 | 0.022 | **0.008** | 0.070 | 0.057 | 0.062 | 0.120 | 0.022 |
| irregular, wobble | 0.089 | 0.066 | 0.089 | 0.088 | 0.185 | 0.060 | 0.064 | 0.144 | 0.089 |
| reversing, wobble | 0.135 | 0.090 | 0.135 | 0.137 | 0.195 | 0.086 | **0.081** | 0.486 | 0.135 |

What this shows, and what it does not:

* With steady rotation the period-matched window is the most accurate, as moma's documentation says. Fixed and auto windows equal it only because the nominal period (40) gives an odd window of 39 or 41 that is close to the period.
* For irregular and reversing rotation, no mean-based window is best; midrange and coverage are. This matches moma's own synthetic benchmark.
* **The consensus matched the fixed/auto window in every row and was never the best.** `consensus_axis` weights estimators by agreement with the others; the likely reason (not separately verified) is that the several window-mean methods agree with each other and outvote the better one. It is a guard against one bad estimator, not a way to beat the best one.
* `refined` (the joint fit) was the worst on this benchmark and failed badly on reversing rotation (0.486). It is not a general upgrade over the plain windows.
* 15% dropout barely changed the result because dropped samples are isolated and linear interpolation across a short gap leaves the window mean almost unchanged. Long gaps are not tested.
* Not tested: other seeds in the table (each row is one seed), other periods or aspect ratios, and larger noise.

`cargo run --release --example helix_recovery` recovers a simulated helix (period 36, radius 2, noise 0.03): period 36.09, radius 2.04 with 1.5% relative spread, and the same period from five pooled tracks.

`cargo run --release --example consensus_automaton` shows the automaton. From random residues it settles into smooth domains: local agreement rises to about 0.93 while the global order stays near 0.14, because the domains disagree with each other. This is a property of the update rule, not a model of anything physical.

## Examples

* `axis_bench`: the table above.
* `helix_recovery`: 3D period, radius and pooled period.
* `consensus_automaton`: the ring-consensus automaton.

## Migrating from 0.x

1.0 is a rewrite around moma 1.0. The old automaton used `moma::core::MomaRing`, which moma 1.0 removed, and the quantum circuit, maze, pathfinding, grid and network graph modules never used moma at all. They and the eight example crates were removed; use 0.4.x if you need them. See the [CHANGELOG](CHANGELOG.md).

## License

MIT OR Apache-2.0.
