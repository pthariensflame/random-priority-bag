# random-priority-bag

A Rust implementation of a random priority bag data structure: a version of a priority queue that allows
duplicates and selects randomly from among the highest-priority elements on each pop, with linear-time
`push`, constant-time `pop` and by-value iteration, and amortized constant time by-reference iteration.
