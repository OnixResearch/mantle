## Why

Evaluation-stream mode owns cooperative cancellation, but the CLI does not connect process interruption signals to that handle. An operator can send `SIGINT` or `SIGTERM` and lose the terminal root summary that the stream contract otherwise guarantees.

## What Changes

- Translate the first supported interruption signal into cooperative evaluation cancellation.
- Continue draining bounded stream records until one canonical cancelled summary is written.
- Treat a repeated interruption as a forced non-success exit instead of ignoring it.
- Keep signal handling local to explicit evaluation-stream mode.
- Add bounded subprocess tests for `SIGINT`, `SIGTERM`, repeated signals, and unchanged no-signal behavior.

## Impact

- **Files**: `src/build_cmd.rs`, the evaluation-stream core, CLI integration tests, stream documentation, and Cairn lifecycle evidence.
- **Testing**: focused core tests, stream CLI subprocess tests, formatting, Clippy, the stream contract checker, Cairn gates, and Tracey coverage.
