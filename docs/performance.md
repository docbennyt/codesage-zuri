# Performance

Performance is a product requirement, but this repository does not publish unmeasured claims.

Initial no-model engineering targets for a roughly 10k LOC Python repository on modest hardware:

- cold index: under 5 seconds;
- no-change incremental check: under 1 second;
- peak engine RAM: comfortably below 250 MB;
- knowledge query: interactive/near-instant.

These are **targets**, not benchmark results.

## What CI proves

The current Rust hardening work has passed tests and optimized release builds on GitHub-hosted Linux, Windows and macOS runners, with strict Clippy on Linux. That is evidence for build portability and deterministic correctness; it is **not** evidence that the 4 GB / 8 GB low-resource product target has been achieved.

## Required representative benchmark gate

Before publishing low-end performance claims, test at minimum:

- a 4 GB Windows or Linux machine with no discrete GPU;
- an 8 GB Windows or Linux machine with no discrete GPU;
- internet disconnected for the no-model benchmark path;
- a small project and a roughly 10k LOC Python project.

Record:

- binary/package size;
- cold and warm startup;
- cold full index time;
- no-change incremental index time;
- changed-file incremental index time;
- peak and idle RAM;
- project SQLite size;
- Knowledge Pack size and query latency;
- explain/trace/review/Vibe Check latency;
- optional-model figures separately from deterministic-core figures.

Benchmark scripts and raw machine details should be committed or attached to releases so performance marketing has receipts.
