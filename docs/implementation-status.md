# v0.1 implementation status

Implemented in the Rust migration branch:

- safe scanner and ignore handling;
- Tree-sitter Python parsing;
- SQLite project index and incremental changed-file updates;
- conservative call resolution;
- Evidence Contract and deterministic findings;
- deterministic explain/trace/concepts/quiz primitives;
- SQLite/FTS5 Knowledge Pack generation and search;
- CLI and Ratatui TUI;
- loopback-only optional model connectivity boundary;
- fixtures and integration tests;
- cross-platform CI/release workflow definitions.

Known validation constraint: the tool execution environment used to author the migration did not include Rust, so local cargo results are intentionally not claimed. CI is the compilation/test authority before merge.
