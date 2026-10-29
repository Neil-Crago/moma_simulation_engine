//! Ring-consensus cellular automaton. Run with
//! `cargo run --release --example consensus_automaton`.

use moma_simulation_engine::ConsensusAutomaton;

fn main() {
    let mut ca = ConsensusAutomaton::torus(64, 12, 20, 1, 42).expect("valid parameters");
    let mut done = 0;
    for checkpoint in [0, 1, 5, 20, 100] {
        ca.run(checkpoint - done);
        done = checkpoint;
        println!("--- after {done} steps: order {:.3}, local agreement {:.3}", ca.order(), ca.local_agreement());
        println!("{}", ca.render());
    }

    let mut noisy = ConsensusAutomaton::torus(64, 12, 20, 1, 42).unwrap().with_noise(0.05).unwrap();
    noisy.run(100);
    println!(
        "with 5% noise after 100 steps: order {:.3}, local agreement {:.3}",
        noisy.order(),
        noisy.local_agreement()
    );
}
