# Architecture

CodeSage Zuri is deliberately **model-optional**.

```text
CLI / TUI
   |
Zuri Core
   |-- scanner
   |-- Tree-sitter Python parser
   |-- SQLite project index / code graph
   |-- deterministic review signals
   |-- Evidence Contract
   |-- SQLite FTS5 Knowledge Pack
   |-- tutor/quiz primitives
   `-- optional local model boundary
```

## Why Rust

The runtime needs low overhead, predictable deployment and strong safety properties on machines where shipping a Python/Node environment is undesirable. Rust also fits Tree-sitter and SQLite directly and can later be reused by Tauri without creating a second backend.

## Why one core crate initially

The design brief proposed several crates, but v0.1 deliberately avoids architecture theatre. Parser/index/review/knowledge modules have explicit boundaries inside `zuri-core`; they can be split once compile-time, ownership or independent-versioning needs justify it.

## Index

Indexes live in an OS-specific application cache keyed by a BLAKE3 hash of the canonical project path. Incremental indexing compares content hashes, reparses changed Python files, removes deleted files in one transaction, then rebuilds call resolution/findings.

## Static uncertainty

Python is dynamic. Zuri currently resolves a direct identifier call only when exactly one indexed symbol has that name. Attribute calls and ambiguous names remain unresolved. This is intentionally conservative.

## Trust boundary

Repositories are untrusted data. Normal indexing never imports or executes them. Knowledge Packs are data-only SQLite files. Local model text is never allowed to manufacture evidence records.
