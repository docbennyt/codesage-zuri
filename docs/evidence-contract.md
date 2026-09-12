# Zuri Evidence Contract

Evidence category and confidence are separate dimensions.

| Category | Meaning |
|---|---|
| `FACT` | Direct parser/source/index relationship |
| `DOCUMENTED` | Supported by a sourced local Knowledge Pack entry |
| `INFERENCE` | Static heuristic or probable interpretation |
| `MODEL` | Optional generated explanation, never authoritative evidence |

A high-confidence inference is still not a fact. A low-severity finding can still be a high-confidence fact.

Every deterministic finding has a rule ID, severity, confidence, evidence category and source location. Finding IDs are derived from rule + path + line so unchanged findings remain stable in common cases.

The local-model layer may eventually rewrite an evidence bundle into friendlier prose, but Zuri Core remains responsible for attaching evidence and locations.
