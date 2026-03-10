# random-priority-bag

A Rust implementation of a random priority bag data structure: a version of a priority queue that allows
duplicates and selects randomly from among the highest-priority elements on each pop.
Features efficient iteration and core functionality that tends to take time either constant or linear in
the number of distinct priorities present, rather than in the number of total elements.
