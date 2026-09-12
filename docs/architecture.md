# Architecture

CodeSage Zuri is deliberately **model-optional**.

```text
CLI / TUI
   |
Zuri Core
   |-- scanner
   |-- Tree-sitter Python parser
   |-- conservative Python resolver
   |-- SQLite project index / code graph
   |-- deterministic review signals
   |-- Evidence Contract
   |-- local benchmark harness
   |-- SQLite FTS5 Knowledge Pack
   |-- tutor/quiz primitives
   `-- optional local model boundary
```

## Why Rust

The runtime needs low overhead, predictable deployment and strong safety properties on machines where shipping a Python/Node environment is undesirable. Rust also fits Tree-sitter and SQLite directly and can later be reused by Tauri without creating a second backend.

## Why one core crate initially

The design brief proposed several crates, but v0.1 deliberately avoids architecture theatre. Parser/resolver/index/review/benchmark/knowledge modules have explicit boundaries inside `zuri-core`; they can be split once compile-time, ownership or independent-versioning needs justify it.

## Index

Indexes live in an OS-specific application cache keyed by a BLAKE3 hash of the canonical project path. Incremental indexing compares content hashes, reparses changed Python files, removes deleted files in one transaction, then rebuilds call resolution/findings.

## Python call resolution

Python is dynamic, so Zuri must distinguish a useful static relationship from runtime certainty.

The resolver no longer links a call merely because a same-named symbol happens to be globally unique in the repository. It prefers narrower evidence:

1. lexical/same-file relationships;
2. explicitly imported repository symbols, including aliases;
3. module-qualified repository calls derived from imports;
4. `self` / `cls` method relationships as probable, not proven.

Call edges therefore have three states:

- `Resolved` — Zuri found a deterministic static repository relationship under the current resolver rules;
- `Probable` — the relationship is useful but depends on Python conventions/dynamic behavior and must be treated as inference;
- `Unresolved` — Zuri preserves the call expression but does not claim a target.

This is intentionally not whole-program Python execution or type inference. Rebinding, monkey-patching, dynamic imports, descriptors, reflection and other runtime behavior can invalidate a static interpretation. Future resolver work should make those limitations narrower without hiding them.

## Evidence propagation

Resolution state is preserved through Explain, Trace, Vibe Check and model-facing Evidence Bundles. A `Probable` edge is `INFERENCE`; finding a plausible target does not promote it to `FACT`. An unresolved call can still contribute the factual statement that the source contains that call expression while its runtime target remains unknown.

## Local benchmarking

`zuri benchmark` measures Zuri Core on the machine that actually runs it. The harness records cold index time, repeated no-change incremental indexing, symbol lookup, Knowledge Pack search, project-index size and platform metadata. Linux additionally reports process high-water RSS from `/proc/self/status`.

The harness does not execute analysed project code and does not use a model or network service. GitHub CI results remain portability/correctness evidence, not proof of 4 GB / 8 GB performance.

## Trust boundary

Repositories are untrusted data. Normal indexing never imports or executes them. Knowledge Packs are data-only SQLite files. Local model text is never allowed to manufacture evidence records.
