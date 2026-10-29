# Changelog

## 1.0.0

Rewrite around moma 1.0, which removed `MomaRing` and `OriginStrategy` (the only moma types this crate used).

### Breaking: removed
- `circuit`, `gates`, `qubit` (quantum simulator), `maze`, `pathfinding`, `grid`, `network_graph`, which never used moma, and the eight example crates under `examples/`. Use 0.3.x/0.4.x if you need them.
- `CellularAutomaton` and `Moma2dAutomaton`, which depended on `moma::core`. Replaced by `ConsensusAutomaton`.
- The `rand`, `num-complex` and `ordered-float` dependencies. The only dependency is `moma` 1.0.

### Added
- `track`: `SpiralSpec` and `HelixSpec` generate tracks with known true axis and phase (fixed, drifting, random-walk or wobbling axis; steady, irregular or reversing rotation; ellipticity, noise, dropped samples). Invalid specs return `Err`.
- `evaluation`: `Estimator` wraps the axis methods of `moma::spiral`; `evaluate`, `consensus` and `axis_error` score them against the truth.
- `automaton::ConsensusAutomaton`: 1D and toroidal 2D automaton using `moma::consensus::ring_consensus`, with seeded runs, self weight, noise, `order` and `local_agreement`.
- `rng::Rng`: deterministic SplitMix64, so runs do not change with `rand` versions.
- 27 unit and doc tests; examples `axis_bench`, `helix_recovery`, `consensus_automaton`.

### Changed
- License is now `MIT OR Apache-2.0`, matching moma and the licence files already in the repository.
