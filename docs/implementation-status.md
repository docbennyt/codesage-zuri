# v0.1 implementation status

Implemented in the Rust core and hardening work:

- safe scanner and ignore handling;
- Tree-sitter Python parsing;
- SQLite project index and incremental changed-file updates;
- conservative call resolution;
- Evidence Contract and deterministic findings;
- deterministic explain, trace, concepts and project-aware quiz primitives;
- `zuri init` with local `.zuri.toml` configuration;
- SQLite/FTS5 Knowledge Pack generation, validated atomic installation and safe search;
- deterministic `quiz --topic` backed by documented local knowledge;
- deterministic Vibe Check readiness/concept/finding report;
- Evidence Bundle construction with bounded source snippets;
- optional loopback-only OpenAI-compatible completion provider used by `explain --model`;
- CLI and Ratatui TUI;
- fixtures and regression tests covering incremental indexing, evidence classes, caller identity, async/decorators, knowledge and Vibe Check;
- committed Cargo lockfile;
- cross-platform CI/release workflow definitions.

## Validation

GitHub Actions is the Rust compilation/test authority for this work. A hardening checkpoint passed tests and optimized release builds on Linux, Windows and macOS, with strict Clippy passing on Linux. The final branch gate additionally enforces committed rustfmt output and locked dependency resolution.

## Not yet claimed

The 4 GB / 8 GB low-resource product target has not yet been validated on representative hardware. GitHub-hosted runners are useful for correctness and portability, but they are not evidence for low-end RAM or latency claims. Representative hardware benchmarking remains a release gate.
