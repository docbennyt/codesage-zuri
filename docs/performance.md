# Performance

Performance is a product requirement, but this repository does not publish unmeasured claims.

Initial no-model engineering targets for a roughly 10k LOC Python repository on modest hardware:

- cold index: under 5 seconds;
- no-change incremental check: under 1 second;
- peak engine RAM: comfortably below 250 MB;
- knowledge query: interactive/near-instant.

These are **targets**, not benchmark results.

The implementation environment used for the initial migration did not contain a Rust toolchain, so no fabricated local benchmark numbers are recorded. GitHub CI is the compilation/test authority for this migration; representative 4 GB and 8 GB hardware benchmarks are the next release gate.

Measure at minimum cold/warm startup, cold/incremental index, peak/idle RAM, database size and Knowledge Pack query latency.
