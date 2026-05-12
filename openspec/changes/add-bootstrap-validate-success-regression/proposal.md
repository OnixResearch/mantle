## Why

`crunch bootstrap validate` is now the shared evidence runner for bootstrap runtime-validation work, but its CLI regression coverage only proves preflight failure. A passing build path must prove that logs, summary JSON, Markdown evidence, and status rendering stay stable.

## What Changes

- **Add**: Add a success-path CLI regression for `crunch bootstrap validate` using a minimal derivation and temporary store/state/evidence directories.
- **Assert**: Assert `validation-summary.json`, `validation-summary.md`, `build.stdout.log`, `build.stderr.log`, and `doctor.json` are written with `passed` status.
- **Keep**: Keep the test hermetic and small enough for normal local test runs.

## Capabilities

### New Capabilities
- `bootstrap-validate-evidence-regression`: Add bootstrap validate success-path regression.

## Impact

- **Files**: `tests/bootstrap_validate_cli.rs`, `src/bootstrap_validate.rs` only if a defect is found.
- **APIs**: No public API change unless implementation tasks discover a necessary narrow seam.
- **Dependencies**: No new default dependency expected.
- **Testing**: Each task records the smallest relevant command or evidence artifact.
