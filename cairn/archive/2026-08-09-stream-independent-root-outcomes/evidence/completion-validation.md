# Completion validation

Question: Do the original baseline rails and the focused Phase 4 completion rails still behave as required?

Inspected evidence:

- Phase 1 baseline commands
- Phase 4 core, pipeline, shell, contract, and CLI tests
- Pueue tasks `17006`, `17008`, `17030`, `17039`, `17043`, and `17046`

Results:

- Pueue task `17006` reran the original baseline suites. It passed 82 `crunch-eval` tests, 37 pipeline unit tests, 19 pipeline integration tests, 22 evaluator CLI integration tests, and 14 evaluator-budget CLI tests. Four pipeline determinism probes remained explicitly ignored.
- Pueue task `17043` passed changed-file Rust formatting, the `wasm32-unknown-unknown` core check, the stream-contract self-test, two positive stream fixtures, five negative stream fixtures, the machine-contract self-test, and `git diff --check`.
- Pueue task `17046` passed focused Clippy for the pure stream core, pipeline, and Mantle targets with dependency linting disabled and warnings denied.
- The focused pure-core Tiger Style check passed in task `16969`.
- The pipeline Tiger Style rail reached only seven existing findings outside the changed evaluation-stream path after all new findings were repaired.
- The Mantle Tiger Style rail stopped on existing root-library findings before it checked the binary target.
- The full machine-contract inventory remains red on existing unclassified root JSON producers. Task `17030` proves that `src/evaluation_stream_output.rs` is now classified and is absent from that blocker list.
- The dedicated evaluation-stream contract rail is green and remains the authority for multi-record NDJSON framing.

Decision: V1 and V2 are complete. Current evidence proves the changed surfaces and records the bounded repository-wide blockers without claiming a full repository pass.

Owner: `stream-independent-root-outcomes`.

Next action: Run pinned Cairn validation, lifecycle gates, and Tracey coverage before sync and archive.
