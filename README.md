# CodeSage Zuri

**Offline Code Literacy Engine**

> Understand the code before you trust or change it.

**AI can write the code. Zuri helps you understand it.**

CodeSage Zuri is a local-first static-analysis, code-understanding and learning tool designed to remain useful without an internet connection, API key, GPU or LLM. The v0.1 foundation focuses deeply on Python and constrained computers rather than pretending to support every language.

## What Zuri does

Zuri indexes a Python repository, extracts structural facts with Tree-sitter, stores a local code graph in SQLite, surfaces deterministic review findings, explains symbols from evidence, traces statically resolvable calls, searches a local source-backed Python Knowledge Pack, identifies programming concepts and generates project-aware learning checks.

**Inspect → Understand → Verify → Learn → Change**

## What Zuri is not

Zuri is not an autonomous coding agent, cloud chatbot wrapper or IDE replacement. It does not execute the project it analyses. Optional local-model support is an explanation layer over Zuri evidence, not the source of truth.

## v0.1 features

- Rust-native `zuri` CLI and Ratatui terminal UI.
- `.gitignore`-aware project scanner with binary/size filtering.
- Python parsing with Tree-sitter.
- Incremental SQLite project index using content fingerprints.
- Symbols, imports, calls, structural metrics and programming concepts.
- Explicit resolved vs unresolved call edges; ambiguous calls remain unresolved.
- Evidence Contract: `FACT`, `DOCUMENTED`, `INFERENCE`, `MODEL`.
- Deterministic review findings with stable IDs.
- `init`, `index`, `status/map`, `symbols`, `explain`, `trace`, `review`, `why`, `concepts`, `search`, `learn`, `quiz`, `vibe-check`, `doctor` and `tui`.
- Local SQLite/FTS5 Python Core Knowledge Pack with provenance.
- Knowledge Pack validation, automatic repair and atomic installation.
- Project-aware quizzes plus `quiz --topic` from documented local knowledge.
- Deterministic Vibe Check for "what should I understand before changing this code?".
- Optional local OpenAI-compatible model completion through a bounded Evidence Bundle.
- Loopback-only model endpoints by default.
- No telemetry, account or cloud upload.

## Build

Install stable Rust, then:

```bash
cargo build --release --locked
```

The binary is produced from `apps/zuri-cli` as `zuri`. No Python or Node.js runtime is required for CLI/TUI execution.

## First run

```bash
zuri init .
zuri index .
zuri status
zuri vibe-check
```

`zuri init` creates a small `.zuri.toml` project configuration and provisions the built-in Python knowledge pack locally. The repository index itself stays in the operating system application cache rather than adding analysis databases to the codebase.

## Understand a project

```bash
zuri map
zuri symbols auth
zuri explain auth.py::authenticate_user
zuri trace auth.py::authenticate_user --depth 2
zuri review
zuri concepts auth.py::authenticate_user
zuri why <finding-id>
```

## Learn offline

```bash
zuri search "mutable default"
zuri learn mutable-defaults
zuri quiz --topic mutable-defaults
zuri quiz auth.py::authenticate_user
```

Project quizzes are generated from indexed repository facts. Topic quizzes come from installed Knowledge Packs and are labeled `DOCUMENTED`.

## Optional local model

The deterministic engine works without a model. To opt in to a local OpenAI-compatible runtime:

```bash
zuri model configure --endpoint http://127.0.0.1:8080 --model <local-model-name>
zuri model test
zuri explain auth.py::authenticate_user --model
```

`--model` sends a compact Evidence Bundle containing Zuri-derived facts, documented knowledge, explicit inferences and bounded source context. Repository text is treated as untrusted data. Generated wording is always `MODEL` evidence and cannot create authoritative Zuri facts.

Important machine-readable commands support `--json`.

## Evidence Contract

Zuri separates what it knows from what it suspects:

- **FACT** — directly derived from source syntax or deterministic repository relationships.
- **DOCUMENTED** — supported by an installed, sourced Knowledge Pack entry.
- **INFERENCE** — useful static heuristic that is not guaranteed.
- **MODEL** — optional generated wording/interpretation; never promoted to fact.

See [`docs/evidence-contract.md`](docs/evidence-contract.md).

## Privacy & safety

Normal analysis is static. Opening a repository does **not** import project modules, execute tests, install dependencies or run setup scripts. Project indexes live in the operating system's application cache rather than arbitrary repositories. Optional model calls are disabled by default and v0.1 rejects non-loopback model hosts.

See [`docs/privacy.md`](docs/privacy.md) and [`docs/model-integration.md`](docs/model-integration.md).

## Hardware goals

The design target is a genuinely useful no-model **ECO** experience on 4 GB systems and ordinary CPUs. The repository deliberately does **not** claim that target has been met until representative 4 GB and 8 GB hardware benchmarks are published. GitHub-hosted CI validates correctness and portability, not low-end performance.

See [`docs/performance.md`](docs/performance.md).

## Architecture

The v0.1 workspace intentionally keeps crate count small:

- `crates/zuri-core` — scanner, parser, index/store, evidence, review, knowledge, tutor primitives and model boundary.
- `apps/zuri-cli` — CLI and Ratatui TUI.
- `crates/zuri-core/assets/python_core.json` — built-in Python Core pack source content/provenance.
- `fixtures/python` — deterministic test/example projects.

See [`docs/architecture.md`](docs/architecture.md).

## Validation

The hardening branch is required to pass:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

CI runs tests and release builds on Linux, Windows and macOS. Strict formatting and Clippy run on Linux.

## Roadmap

1. Benchmark representative 4 GB and 8 GB machines and publish receipts.
2. Deepen Python lexical-scope, import and call resolution.
3. Add reference edges and stronger deterministic side-effect/change-impact facts.
4. Add Knowledge Pack integrity hashes/signatures.
5. Add C as the next deeply supported language.
6. Add JS/TS and PHP only after adapter quality gates exist.
7. Build the optional Tauri v2 + Svelte desktop client over the same Rust core.

## Licensing

No project licence has been selected yet. This remains an explicit owner decision; no licence is implied by the repository being public.

## Status

CodeSage Zuri should be judged by deterministic behavior, evidence quality, offline usefulness and published benchmarks—not unsupported AI marketing claims.
