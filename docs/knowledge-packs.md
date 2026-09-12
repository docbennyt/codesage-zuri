# Knowledge Packs

Zuri knowledge is data, not executable application code.

The Python Core pack is represented in source by a manifest plus original editorial summaries with explicit source metadata. At runtime Zuri materialises a single SQLite `.zpk` file containing manifest metadata, entries, provenance and an FTS5 index queried with BM25 ranking.

The original `white_byte.py` prototype is not treated as an authoritative corpus. Its useful topic areas were rewritten into source-backed summaries.

Packs are independently versionable and can eventually be distributed by download, USB or LAN without requiring an account. Pack integrity signatures/checksums are a future hardening step.
