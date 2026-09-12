# v0.1 implementation status

Implemented in the Rust core and hardening work:

- safe scanner and ignore handling;
- Tree-sitter Python parsing;
- SQLite project index and incremental changed-file updates;
- conservative call resolution using lexical/same-file and explicit import evidence rather than repository-global name guessing;
- imported aliases and module-qualified repository call resolution;
- explicit `Resolved`, `Probable` and `Unresolved` call states;
- `self` / `cls` method relationships classified as probable rather than authoritative;
- probable call edges preserved as `INFERENCE` in Evidence Bundles;
- Evidence Contract and deterministic findings;
- deterministic explain, trace, concepts and project-aware quiz primitives;
- `zuri init` with local `.zuri.toml` configuration;
- SQLite/FTS5 Knowledge Pack generation, validated atomic installation and safe search;
- deterministic `quiz --topic` backed by documented local knowledge;
- deterministic Vibe Check readiness/concept/finding report with separate probable/unresolved call counts;
- local `zuri benchmark` harness for cold indexing, no-change incremental indexing, symbol lookup, Knowledge Pack search and index size, with Linux peak-RSS reporting;
- Evidence Bundle construction with bounded source snippets;
- optional loopback-only OpenAI-compatible completion provider used by `explain --model`;
- CLI and Ratatui TUI;
- fixtures and regression tests covering incremental indexing, evidence classes, caller identity, async/decorators, knowledge, Vibe Check, import aliases, probable self-method links, false global-name matches and the benchmark harness;
- committed Cargo lockfile;
- cross-platform CI/release workflow definitions.

## Validation

The resolver/benchmark implementation passed `cargo fmt`, strict Clippy with `-D warnings`, and the full workspace test suite on the Linux implementation gate before being committed to the feature branch. The feature PR must still pass the normal immutable CI matrix before merge: locked tests and optimized release builds on Linux, Windows and macOS, with committed rustfmt and strict Clippy on Linux.

## Not yet claimed

The 4 GB / 8 GB low-resource product target has not yet been validated on representative hardware. The benchmark harness makes that validation reproducible, but it does not replace the machines themselves. GitHub-hosted runners remain correctness/portability evidence rather than low-end RAM or latency evidence.

Python resolution is also not whole-program runtime resolution. Assignment rebinding, star imports, monkey-patching, reflection, descriptors, dynamic imports and other runtime behavior remain deliberately outside the current deterministic proof boundary.
