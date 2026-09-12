# Optional local model integration

The model is not Zuri's analysis engine.

v0.1 stores explicit local model configuration and supports an OpenAI-compatible loopback endpoint. The current endpoint parser accepts `http://127.0.0.1:<port>` and `http://localhost:<port>` and rejects non-loopback hosts by default.

## Provider boundary

`ModelProvider` defines the optional completion boundary. The current `LocalOpenAiProvider` can:

- probe `/v1/models`;
- send completion requests to `/v1/chat/completions`;
- remain disabled without affecting any deterministic Zuri capability.

Zuri does not require Ollama or bundle a model. Any local runtime that exposes the expected OpenAI-compatible loopback API can sit behind this boundary.

## Evidence Bundle

`zuri explain <target> --model` first runs the normal deterministic explanation path. Only then does Zuri construct an Evidence Bundle containing:

- verified repository facts;
- relevant source-backed Knowledge Pack summaries;
- explicit static inferences;
- a bounded source snippet for the selected symbol.

Repository text is treated as untrusted data rather than instructions. The model system prompt requires it to explain only supplied evidence, preserve uncertainty and avoid inventing source locations, rule IDs, dependencies or behavior.

Model output is labeled `MODEL`. It does not modify the SQLite evidence graph and cannot create `FACT` or `DOCUMENTED` records.

## Privacy boundary

Model enhancement is opt-in and disabled by default. v0.1 does not intentionally support remote model endpoints. Normal indexing, review, explanation, trace, knowledge search, quizzes and Vibe Check require no model and no network connection.
