## Why

Provider fixed-point proof bundles now participate in the explicit full-release global reproducibility universe. The verifier can rebase copied bundles, but newly generated proof metadata still records absolute stage artifact paths from the original scratch directory. That makes portable evidence harder to audit and risks accidental dependence on publisher-local directories.

## What Changes

- Generate provider fixed-point proof metadata with bundle-local relative paths for proof-owned stage directories, receipts, logs, status files, execution directory, and stage binaries.
- Keep external provenance paths, such as source root and source-built toolchain/provider paths, explicit instead of pretending they are bundle-local artifacts.
- Preserve verifier compatibility with older absolute-path proof bundles while proving new metadata is relative at generation time.

## Impact

- **Files**: provider fixed-point summary/preflight generation, focused unit tests, verification-evidence spec delta, archived Cairn evidence.
- **Testing**: focused provider fixed-point summary/verifier tests, formatting, diff check, Cairn validation/gates/archive.
