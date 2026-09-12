# CodeSage Zuri

**Offline Code Literacy Engine**

> Understand the code before you trust or change it.

**AI can write the code. Zuri helps you understand it.**

CodeSage Zuri is a local-first static-analysis, code-understanding and learning tool designed to remain useful without an internet connection, API key, GPU or LLM. The v0.1 foundation focuses deeply on Python and constrained computers rather than pretending to support every language.

## What Zuri does

Zuri indexes a Python repository, extracts structural facts with Tree-sitter, stores a local code graph in SQLite, surfaces deterministic review findings, explains symbols from evidence, resolves conservative lexical/import relationships, traces calls with uncertainty labels, searches a local source-backed Python Knowledge Pack, identifies programming concepts and generates project-aware learning checks.

**Inspect → Understand → Verify → Learn → Change**

## What Zuri is not

Zuri is not an autonomous coding agent, cloud chatbot wrapper or IDE replacement. It does not execute the project it analyses. Optional local-model support is an explanation layer over Zuri evidence, not the source of truth.

## v0.1 features

- Rust-native `zuri` CLI and Ratatui terminal UI.
- `.gitignore`-aware project scanner with binary/size filtering.
- Python parsing with Tree-sitter.
- Incremental SQLite project index using content fingerprints.
- Symbols, imports, calls, structural metrics and programming concepts.
- Conservative Python call resolution using lexical scope and repository imports instead of repository-global name guessing.
- Import aliases and module-qualified repository calls where the relationship can be established statically.
- Explicit `Resolved`, `Probable` and `Unresolved` call edges. `self` / `cls` method links are deliberately `Probable` because Python remains dynamic.
- Evidence Contract: `FACT`, `DOCUMENTED`, `INFERENCE`, `MODEL`.
- Probable call relationships remain `INFERENCE` in model-facing Evidence Bundles; they are never promoted to FACT just because a target was found.
- Deterministic review findings with stable IDs.
- `init`, `index`, `status/map`, `symbols`, `explain`, `trace`, `review`, `why`, `concepts`, `search`, `learn`, `quiz`, `vibe-check`, `benchmark`, `doctor` and `tui`.
- Local SQLite/FTS5 Python Core Knowledge Pack with provenance.
- Knowledge Pack validation, automatic repair and atomic installation.
- Project-aware quizzes plus `quiz --topic` from documented local knowledge.
- Deterministic Vibe Check for "what should I understand before changing this code?".
- Local benchmark harness for cold indexing, no-change incremental indexing, symbol lookup, knowledge search and index size; Linux also reports process peak RSS (`VmHWM`).
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

Call edges shown by Explain/Trace carry their resolution state. A probable relationship is useful context, not a claim of runtime certainty.

## Learn offline

```bash
zuri search "mutable default"
zuri learn mutable-defaults
zuri quiz --topic mutable-defaults
zuri quiz auth.py::authenticate_user
```

Project quizzes are generated from indexed repository facts. Topic quizzes come from installed Knowledge Packs and are labeled `DOCUMENTED`.

## Measure this machine

```bash
zuri benchmark . --rounds 5
zuri benchmark . --rounds 10 --json
```

The benchmark command is intentionally local and model-free. It rebuilds Zuri's cache for the cold-index sample, then measures repeated no-change incremental indexing, symbol lookup and Knowledge Pack search. It never executes the analysed project.

A benchmark result describes **the machine it was run on**. GitHub-hosted runner numbers must not be presented as proof that Zuri meets its 4 GB / 8 GB target. Representative low-resource machines are still required before publishing those claims.

See [`docs/performance.md`](docs/performance.md).

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

- **FACT** — directly derived from source syntax or a deterministic repository relationship.
- **DOCUMENTED** — supported by an installed, sourced Knowledge Pack entry.
- **INFERENCE** — useful static heuristic or probable relationship that is not guaranteed.
- **MODEL** — optional generated wording/interpretation; never promoted to fact.

See [`docs/evidence-contract.md`](docs/evidence-contract.md).

## Privacy & safety

Normal analysis is static. Opening a repository does **not** import project modules, execute tests, install dependencies or run setup scripts. Project indexes live in the operating system's application cache rather than arbitrary repositories. Optional model calls are disabled by default and v0.1 rejects non-loopback model hosts.

See [`docs/privacy.md`](docs/privacy.md) and [`docs/model-integration.md`](docs/model-integration.md).

## Hardware goals

The design target is a genuinely useful no-model **ECO** experience on 4 GB systems and ordinary CPUs. The repository deliberately does **not** claim that target has been met until representative 4 GB and 8 GB hardware benchmarks are published. GitHub-hosted CI validates correctness and portability, not low-end performance.

## Architecture

The v0.1 workspace intentionally keeps crate count small:

- `crates/zuri-core` — scanner, parser, resolver, index/store, evidence, review, benchmark, knowledge, tutor primitives and model boundary.
- `apps/zuri-cli` — CLI and Ratatui TUI.
- `crates/zuri-core/assets/python_core.json` — built-in Python Core pack source content/provenance.
- `fixtures/python` — deterministic test/example projects.

See [`docs/architecture.md`](docs/architecture.md).

## Validation

Changes are required to pass:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

CI runs tests and release builds on Linux, Windows and macOS. Strict formatting and Clippy run on Linux.

## Roadmap

1. Run and publish the new benchmark protocol on representative 4 GB and 8 GB Windows/Linux machines.
2. Deepen Python resolution further with assignment/rebinding awareness, star-import handling and stronger relative-package semantics without inventing certainty.
3. Add reference edges and stronger deterministic side-effect/change-impact facts.
4. Add Knowledge Pack integrity hashes/signatures.
5. Add C as the next deeply supported language.
6. Add JS/TS and PHP only after adapter quality gates exist.
7. Build the optional Tauri v2 + Svelte desktop client over the same Rust core.

## Licensing

No project licence has been selected yet. This remains an explicit owner decision; no licence is implied by the repository being public.

## Status

CodeSage Zuri should be judged by deterministic behavior, evidence quality, offline usefulness and published benchmarks—not unsupported AI marketing claims.
