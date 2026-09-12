# Roadmap

## v0.1 foundation — implemented

Deep Python vertical slice: safe scanning, deterministic indexing/review/explain/trace, source-backed knowledge search, project-aware learning primitives, CLI/TUI, project initialization, deterministic Vibe Check, topic quizzes, and optional evidence-bounded local-model explanation.

## Next release gates

- benchmark representative 4 GB and 8 GB Windows/Linux machines and publish cold/warm latency, RAM and index-size receipts;
- strengthen Python lexical-scope, import and call resolution;
- add reference edges and safer deterministic side-effect/change-impact facts;
- make Vibe Check more instructional with comprehension checks before suggested code changes;
- add Knowledge Pack integrity hashes/signatures and upgrade/migration policy;
- improve TUI navigation/search without making the TUI a second analysis engine.

## Language expansion

C is the next candidate because its static semantics allow useful deterministic analysis on constrained hardware. JS/TS and PHP follow only when adapter quality gates exist. "File extension recognised" must never be marketed as language support.

## Desktop

Tauri v2 + Svelte + TypeScript, reusing the same Rust core. Desktop polish must not compromise the terminal/offline baseline.

## Model work

Model support remains optional. Future work may add better runtime discovery and adapters for common local servers, but all providers must consume Zuri Evidence Bundles and remain unable to mint authoritative evidence records.
