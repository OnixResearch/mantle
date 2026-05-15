## Why

Mantle can now produce deterministic-build proof receipts from repeated clean proof runs, but those proof runs currently execute an operator-supplied rebuild command directly. Fresh output/store directories and BLAKE3 matches are not enough to prove deterministic behavior if the rebuild recipe can read ambient host files, use undeclared host tools, or reach the network.

## What Changes

- Require deterministic proof runs to execute through a recorded sandbox envelope rather than a direct host process.
- Deny network access by default for deterministic proof runs.
- Bind only the release evidence bundle, the selected rebuild recipe/tool path, a fresh output directory, and a fresh proof store directory.
- Record a canonical sandbox profile digest in deterministic proof receipts.
- Fail closed when the sandbox executor is missing, unsupported, bypassed, or when the recipe requires undeclared host access.

## Capabilities

### Modified Capabilities
- `release-verification-tech`: deterministic proof receipts become evidence of sandboxed proof execution, not only repeated host command execution.
- `build-pipeline`: deterministic proof orchestration gains a dedicated proof-sandbox envelope distinct from general derivation sandboxing.

## Impact

- **Files**: `src/release_reproducibility.rs`, `src/main.rs`/release CLI as needed, `crates/crunch-release-core/src/determinism.rs`, README, and tests.
- **APIs**: deterministic proof receipt schema gains sandbox profile identity/evidence or a versioned successor.
- **Testing**: unit tests for sandbox profile validation and fail-closed missing/unsupported executor; CLI tests proving deterministic proof runs use the sandbox wrapper and reject host-only recipes.
