## Why

Unison transcripts make documentation, bug reproductions, and tool workflows executable markdown. Mantle already depends on evidence receipts and OpenSpec validation, but examples and repros can still drift from actual CLI behavior. Executable Mantle transcripts would let docs, bug reports, and release-proof walkthroughs double as deterministic tests.

## What Changes

- **Transcript format**: Markdown fenced blocks for Mantle commands, expected output fragments, expected errors, hidden setup, and isolated state.
- **Runner**: A local transcript runner that executes against temporary stores/state by default.
- **Quality gate integration**: Docs and bug repro transcripts can be included in maintained checks.

## Capabilities

### New Capabilities
- `mantle-transcripts`: executable markdown workflows for docs, tests, and bug repros.

### Modified Capabilities
- `quality-gates`: optional transcript check entrypoint.
- `operator-diagnostics`: transcript failures emit typed diagnostics and saved output.

## Impact

- **Files**: transcript parser/runner, scripts or CLI subcommand, docs examples, test fixtures.
- **APIs**: additive runner surface; no change to core build semantics.
- **Testing**: positive transcript, expected-error transcript, hidden setup, state isolation, and stale-output tests.
