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

## Call edges

Python call targets have their own resolution state and that state must survive every presentation layer:

- `Resolved` means the current deterministic resolver can establish a static repository relationship from lexical/import evidence.
- `Probable` means Zuri found a useful likely target but Python's dynamic behavior prevents the relationship from being treated as fact. Probable edges map to `INFERENCE` when an Evidence Bundle is constructed.
- `Unresolved` means the call expression is known from source but Zuri does not claim a target. The existence/location of the expression is still a fact; its runtime target is not.

A model must never turn `Probable` or `Unresolved` into `FACT`. Likewise, a resolver improvement may change an edge's state only because deterministic repository evidence improved—not because a language model suggested a target.

## Model boundary

The local-model layer may rewrite an Evidence Bundle into friendlier prose, but Zuri Core remains responsible for attaching evidence, resolution state and locations. Model output is always `MODEL` evidence and cannot mint rule IDs, source locations or authoritative call relationships.
