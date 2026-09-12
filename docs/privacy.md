# Privacy and repository safety

Default Zuri operation is local.

- No account.
- No telemetry.
- No analytics endpoint.
- No cloud upload.
- No API key.
- No network is required for indexing, review, explanation, trace, knowledge search or quizzes.

Zuri reads eligible repository files subject to ignore rules and size/binary filtering. It does not automatically import Python modules, run tests, install packages, execute setup scripts, invoke project hooks or run generated binaries.

Indexes are stored under the operating system's CodeSage Zuri application cache. The CLI reports the concrete database path through `zuri status`.

## Optional model mode

Model configuration is disabled by default. v0.1 accepts only explicit local `http://localhost` / loopback endpoints for its connectivity probe. The deterministic engine remains available if the endpoint is absent.

Source comments and strings are untrusted repository data, not model instructions.
