# Performance

Performance is a product requirement, but this repository does not publish unmeasured claims.

Initial no-model engineering targets for a roughly 10k LOC Python repository on modest hardware:

- cold index: under 5 seconds;
- no-change incremental check: under 1 second;
- peak engine RAM: comfortably below 250 MB;
- knowledge query: interactive/near-instant.

These are **targets**, not benchmark results.

## Built-in benchmark harness

The CLI now provides a local, deterministic benchmark path:

```bash
zuri benchmark . --rounds 5
zuri benchmark . --rounds 10 --json
```

The harness records:

- platform and architecture;
- available CPU thread count;
- cold full-index time after rebuilding Zuri's cache;
- repeated no-change incremental-index timings and median;
- repeated indexed-symbol lookup timings and median when the project contains symbols;
- repeated built-in Knowledge Pack query timings and median;
- project SQLite index size;
- resolved / probable / unresolved call counts as part of project stats;
- Linux process high-water RSS (`VmHWM`) when `/proc/self/status` is available.

The benchmark path reports `network_used=false` and `model_used=false`. It does not execute analysed project code. Rebuilding the cold index mutates only Zuri's cache, not the repository.

Windows and macOS peak-RSS reporting is not yet built into Zuri Core; use an OS-native memory profiler for those runs and record that measurement beside the JSON output.

## What CI proves

The Rust core has passed tests and optimized release builds on GitHub-hosted Linux, Windows and macOS runners, with strict Clippy on Linux. That is evidence for build portability and deterministic correctness; it is **not** evidence that the 4 GB / 8 GB low-resource product target has been achieved.

Numbers from GitHub-hosted runners may be used to detect regressions, but they must not be marketed as low-resource benchmark results.

## Representative hardware gate

Before publishing low-end performance claims, test at minimum:

- a 4 GB Windows or Linux machine with no discrete GPU;
- an 8 GB Windows or Linux machine with no discrete GPU;
- internet disconnected for the no-model benchmark path;
- a small project and a roughly 10k LOC Python project;
- an optimized `cargo build --release --locked` binary, not a debug build.

For each machine, record:

1. exact OS/version and CPU;
2. installed RAM and whether swap/pagefile is enabled;
3. Zuri commit/release and binary size;
4. target repository commit and approximate Python LOC;
5. `zuri benchmark <repo> --rounds 10 --json` output;
6. changed-file incremental-index time separately by editing/touching one representative Python source file and re-indexing;
7. idle and peak RAM using the built-in Linux figure or an OS-native profiler;
8. `explain`, `trace`, `review` and `vibe-check` latency for a representative target;
9. optional-model figures in a separate section, never mixed with deterministic-core figures.

Run more than once after a reboot where practical. Preserve raw JSON and machine details alongside any summarized table so performance marketing has receipts.

## Publication rule

Do not write "runs on 4 GB" merely because the binary starts. The ECO target is met only if indexing, navigation, review, learning and deterministic explanation remain usable on representative 4 GB hardware with reasonable memory headroom.
