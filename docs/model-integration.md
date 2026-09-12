# Optional local model integration

The model is not Zuri's analysis engine.

v0.1 stores explicit local model configuration and can probe an OpenAI-compatible loopback endpoint at `/v1/models`. For privacy, the current endpoint parser refuses non-loopback hosts by default.

Future completion requests must receive a compact Evidence Bundle produced by Zuri Core containing verified facts, documented knowledge, inferences and bounded source snippets. A model may explain that bundle; it may not invent source locations, rule IDs or evidence records.

Zuri should interoperate with local runtimes rather than requiring Ollama or bundling a model.
