# CodeSage Zuri

**Offline Code Literacy Engine**

> Understand the code before you trust or change it.

**AI can write the code. Zuri helps you understand it.**

CodeSage Zuri is a local-first static-analysis, code-understanding and learning tool designed to remain useful without an internet connection, API key, GPU or LLM. The v0.1 foundation focuses deeply on Python and constrained computers rather than pretending to support every language.

## What Zuri does

Zuri indexes a Python repository, extracts structural facts with Tree-sitter, stores a local code graph in SQLite, surfaces deterministic review findings, explains symbols from evidence, traces statically resolvable calls, searches a local source-backed Python Knowledge Pack, identifies programming concepts and generates simple project-aware quizzes.

**Inspect → Understand → Verify → Learn → Change**

## What Zuri is not

Zuri is not an autonomous coding agent, cloud chatbot wrapper or IDE replacement. It does not execute the project it analyses. Optional local-model support is an enhancement boundary, not the source of truth.

## v0.1 features

- Rust-native `zuri` CLI.
- Ratatui terminal UI.
- `.gitignore`-aware project scanner with binary/size filtering.
- Python parsing with Tree-sitter.
- Incremental SQLite project index.
- Symbols, imports, calls, structural metrics and programming concepts.
- Explicit resolved vs unresolved call edges.
- Evidence Contract: `FACT`, `DOCUMENTED`, `INFERENCE`, `MODEL`.
- Deterministic review findings with stable IDs.
- `explain`, `trace`, `map/status`, `why`, `concepts`, `quiz`.
- Local SQLite/FTS5 Python Core Knowledge Pack.
- Local-only optional model endpoint configuration/probe.
- No telemetry, account or cloud upload.

## Build

Install stable Rust, then:

```bash
cargo build --release
```

The binary is produced from `apps/zuri-cli` as `zuri`. No Python or Node.js runtime is required for CLI/TUI execution.

## Example

```bash
zuri index .
zuri map
zuri symbols auth
zuri explain auth.py::authenticate_user
zuri trace auth.py::authenticate_user --depth 2
zuri review
zuri search "mutable default"
zuri quiz auth.py::authenticate_user
zuri tui
```

Important machine-readable commands support `--json`.

## Evidence Contract

Zuri separates what it knows from what it suspects:

- **FACT** — directly derived from source syntax or deterministic repository relationships.
- **DOCUMENTED** — supported by an installed, sourced Knowledge Pack entry.
- **INFERENCE** — useful static heuristic that is not guaranteed.
- **MODEL** — optional generated wording/interpretation; never promoted to fact.

See [`docs/evidence-contract.md`](docs/evidence-contract.md).

## Privacy & safety

Normal analysis is static. Opening a repository does **not** import project modules, execute tests, install dependencies or run setup scripts. Project indexes live in the operating system's application cache rather than arbitrary repositories.

See [`docs/privacy.md`](docs/privacy.md).

## Hardware goals

The design target is a genuinely useful no-model **ECO** experience on 4 GB systems and ordinary CPUs. Performance claims remain engineering targets until measured on representative hardware. See [`docs/performance.md`](docs/performance.md).

## Architecture

The v0.1 workspace intentionally keeps crate count small:

- `crates/zuri-core` — scanner, parser, index/store, evidence, review, knowledge, tutor primitives and model boundary.
- `apps/zuri-cli` — CLI and Ratatui TUI.
- `packs/python-core` — pack specification/provenance.
- `fixtures/python` — deterministic test/example projects.

See [`docs/architecture.md`](docs/architecture.md).

## Roadmap

1. Make Python analysis deeper and benchmarked.
2. Improve lexical-scope/import/call resolution.
3. Add deterministic Vibe Check/comprehension paths.
4. Add C as the next deeply supported language.
5. Add JS/TS and PHP only after adapter quality gates exist.
6. Build the optional Tauri v2 + Svelte desktop client over the same Rust core.

## Licensing

No project licence has been selected yet. This remains an explicit owner decision; no licence is implied by the repository being public.

## Status

The Rust v0.1 branch should be judged by working deterministic behaviour and CI, not unsupported marketing claims.
