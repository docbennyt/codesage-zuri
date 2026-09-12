# ADR 0001 — Core v0.1 decisions

**Status:** Accepted.

1. **Rust core:** stable Rust owns the application engine; Python/Node/Java/Docker are not runtime requirements.
2. **No-model-first:** indexing, review, explain, trace, knowledge and quizzes remain meaningful with model support absent.
3. **Evidence Contract:** FACT, DOCUMENTED, INFERENCE and MODEL are explicit types; confidence is separate.
4. **Terminal-first:** CLI plus Ratatui TUI is the guaranteed low-resource interface.
5. **Knowledge Packs:** local SQLite `.zpk` + FTS5 rather than hard-coded dictionaries or mandatory embeddings.
6. **Future desktop:** Tauri v2 + Svelte + TypeScript + Vite over the same Rust core; no Electron and no second REST backend.
7. **Language depth over count:** Python first, C next only after Python quality/performance gates; JS/TS and PHP later.
