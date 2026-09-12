# Roadmap

## v0.1 foundation — implemented

Deep Python vertical slice: safe scanning, deterministic indexing/review/explain/trace, source-backed knowledge search, project-aware learning primitives, CLI/TUI, project initialization, deterministic Vibe Check, topic quizzes, optional evidence-bounded local-model explanation, conservative lexical/import call resolution and a local benchmark harness.

## Current release gates

- run the built-in benchmark protocol on representative 4 GB and 8 GB Windows/Linux machines and publish raw receipts;
- deepen Python resolution with assignment/rebinding awareness, star-import handling, more complete relative-package semantics and explicit regression fixtures for each case;
- add reference edges and safer deterministic side-effect/change-impact facts;
- make Vibe Check more instructional with comprehension checks before suggested code changes;
- add Knowledge Pack integrity hashes/signatures and upgrade/migration policy;
- improve TUI navigation/search without making the TUI a second analysis engine.

Resolver quality is measured by reducing false certainty, not by maximizing the number of calls marked resolved. Unknown or dynamic behavior should remain `Unresolved` or `Probable` until deterministic evidence justifies stronger classification.

## Language expansion

C is the next candidate because its static semantics allow useful deterministic analysis on constrained hardware. JS/TS and PHP follow only when adapter quality gates exist. "File extension recognised" must never be marketed as language support.

Python correctness and representative low-resource measurements remain higher priority than adding a second language merely to increase a feature count.

## Desktop

Tauri v2 + Svelte + TypeScript, reusing the same Rust core. Desktop polish must not compromise the terminal/offline baseline.

## Model work

Model support remains optional. Future work may add better runtime discovery and adapters for common local servers, but all providers must consume Zuri Evidence Bundles and remain unable to mint authoritative evidence records.
