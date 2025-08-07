# MOMA Agent Behavioural Analysis

This is an experiment and a necessary step before I move on to the upcoming Network Flow project. The objective was to write the code to conduct the experiments automatically and print a final report, with no visualization required for speed.

## The Experiment: "Agent Personality Profile"

**Hypothesis:** Different MOMA Origin Strategies will produce quantitatively different "personalities" in the agent. We can measure this personality by observing the trade-offs it makes between path efficiency (length) and path complexity (Gowers norm).

## Methodology:

* Define a set of MOMA strategies to test (e.g., PrimeGap, CompositeMass, a Fixed value).
* For each strategy, run the simulation for a fixed number of steps (e.g., 200 iterations).
* In each step, the automaton evolves, and the agent finds a new path.
* Record the path_length and the gowers_norm for each path found.
* After all iterations, calculate the average path length and average Gowers norm for each strategy.
* Present the results in a clear summary table.

## Conclusion 

This report will provide the hard data I need to draw conclusions about how different MOMA strategies influence the agent's behavior, officially completing this phase of the project.
