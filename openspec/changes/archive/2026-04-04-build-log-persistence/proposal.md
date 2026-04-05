## Why

The build-pipeline spec requires:

> Build stdout and stderr MUST be captured and:
> - Displayed to the user on build failure (exit code != 0)
> - Available via `--verbose` on success
> - Stored in a log directory keyed by derivation hash

Current state:

1. **Failure display** — partially works. The bwrap `BuildService` wraps
   build failures in `io::Error`. The error message may contain the build
   log, but it's whatever bwrap chose to include. There's no structured
   capture of stdout/stderr from the sandbox.

2. **Success verbose** — broken. On success, `main.rs` writes
   `"build succeeded: {label}\n"` to the log file. The actual build output
   (compiler warnings, progress messages) is lost. `--verbose` shows
   tracing debug lines, not builder output.

3. **Log persistence** — the log directory is set up
   (`$CRUNCH_LOG_DIR` or `$XDG_STATE_HOME/crunch/logs/`) and a `.log` file
   is written on failure. But the file contains the Rust error message,
   not the raw sandbox output. On success, the log is a one-liner.

The root problem: `BuildService::do_build` returns `BuildResult` with output
nodes but no captured stdout/stderr. The builder's console output goes
to... nowhere useful. We need to capture it.

## What Changes

- Capture sandbox stdout/stderr as structured data in `BuildOutcome`.
- Persist full build logs to `$CRUNCH_LOG_DIR/<drv-hash>.log` for every
  build (success and failure).
- Display build output on failure unconditionally.
- Display build output on success when `--verbose` is set.
- Add `crunch log <drv-hash>` subcommand to retrieve past build logs.

## Capabilities

### New Capabilities
- `build-log-capture`: Capture sandbox stdout/stderr in `BuildOutcome`.
- `build-log-persist`: Write log files keyed by derivation store path hash.
- `build-log-display`: Show logs on failure (always) and success (verbose).
- `build-log-retrieve`: `crunch log` command to read stored logs.

### Modified Capabilities
- `build-execution`: `BuildOutcome` gains a `log: Option<String>` field.

## Impact

- **Files**: `crates/crunch-build/src/orchestrate.rs` (BuildOutcome, log capture),
  `src/main.rs` (log display, new subcommand)
- **APIs**: `BuildOutcome` gets a `log` field
- **Dependencies**: None
- **Testing**: Unit tests for log persistence paths, integration test for
  `crunch log` subcommand
