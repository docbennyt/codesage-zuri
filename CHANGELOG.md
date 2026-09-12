# Changelog

## 0.1.0 — unreleased

- Replaced the executable Python-dictionary prototype architecture with a Rust workspace.
- Added safe project scanning, Tree-sitter Python parsing and an incremental SQLite index.
- Added conservative symbol/call relationships with unresolved ambiguity preserved explicitly.
- Added Evidence Contract labels: `FACT`, `DOCUMENTED`, `INFERENCE`, `MODEL`.
- Added evidence-aware deterministic review, explanations, traces and concept extraction.
- Added stable review finding IDs and initial Python rules for syntax errors, mutable defaults, bare exceptions, dynamic execution, built-in shadowing, literal zero divisors and `subprocess(..., shell=True)` inference.
- Added a local SQLite/FTS5 Python Core Knowledge Pack with provenance, safe query handling, schema validation and atomic repair/install.
- Added project-aware quizzes and documented `quiz --topic` learning.
- Added `zuri init` and deterministic `zuri vibe-check`.
- Added an optional loopback-only OpenAI-compatible `ModelProvider` path using bounded Zuri Evidence Bundles; model output remains non-authoritative.
- Added CLI, Ratatui TUI, fixtures and expanded regression tests.
- Added a committed Cargo lockfile and cross-platform locked CI with formatting, Clippy, tests and release builds.
- Added architecture, Evidence Contract, privacy, performance, knowledge-pack, model-integration and roadmap documentation.
