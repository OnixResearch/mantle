## Why

`src/main.rs` (370 lines) has no tests. The bootstrap command shells out to
`nix-build` and `nix build` with no test coverage — a PATH issue or changed
nix CLI output silently breaks it. Error messages are functional but don't
guide the user toward fixes (e.g., missing bwrap, permission errors on
`/nix/store`, broken seed paths).

Build failure output dumps the raw bwrap/sandbox error without extracting
the builder's stderr, making debugging hard for users.

## What Changes

- Add unit tests for CLI argument parsing and error code mapping.
- Improve build failure messages: extract builder stderr from sandbox logs,
  show the failing command, suggest common fixes.
- Add bootstrap tests using a mock nix-build that returns known paths.
- Structured error output option (`--json` flag) for tooling integration.

## Capabilities

### New Capabilities
- `cli-error-tests`: Unit tests for error classification and exit codes.
- `bootstrap-tests`: Tests for seed.ncl generation with mock nix commands.
- `structured-errors`: `--json` flag emitting machine-readable error output.

### Modified Capabilities
- `build-failure-messages`: Extract and display builder stderr on failure.

## Impact

- **Files**: `src/main.rs`, possibly new `src/errors.rs`
- **APIs**: New `--json` CLI flag
- **Dependencies**: None
- **Testing**: `cargo test -p crunch` covers CLI tests
