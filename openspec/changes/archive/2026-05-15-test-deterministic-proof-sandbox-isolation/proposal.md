## Why

Mantle now executes deterministic release proof runs through a recorded proof sandbox profile, but the regression suite only proves the happy-path wrapper and missing-executor failure. A future refactor could accidentally bind host paths, expose the main rebuild output, or permit network behavior while still recording a supported-looking sandbox profile identity.

## What Changes

- Add explicit OpenSpec requirements that deterministic proof sandbox tests cover isolation-negative evidence, not only wrapper invocation.
- Add regression tests that fail closed when a deterministic proof recipe depends on undeclared host paths.
- Add regression tests that inspect the proof sandbox invocation for network denial and absence of main-output/store reuse.
- Keep the proof claim bounded to sandbox-backed release-artifact evidence; do not broaden it to global Mantle determinism.

## Capabilities

### Modified Capabilities
- `build-pipeline`: deterministic proof sandbox execution must be backed by isolation-regression evidence.
- `release-verification-tech`: deterministic proof receipt acceptance remains fail-closed when sandbox isolation evidence is absent or bypassed.

## Impact

- **Files**: `tests/release_cli.rs`, OpenSpec deltas, possibly `src/release_reproducibility.rs` if the sandbox invocation lacks a checkable invariant.
- **APIs**: no user-facing CLI/API changes expected.
- **Dependencies**: no new runtime dependencies.
- **Testing**: targeted release CLI tests plus OpenSpec validation and formatting.
