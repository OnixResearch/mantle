# rust-topology-self-package-lib-binding

## Why

After compile-time env allowlisting, the clean Mantle self `rust-plan --execute-topology` probe advances to a deterministic internal blocker: the root `mantle` lib unit waits for dependency package `path+file:///home/brittonr/git/mantle#0.1.0`, which is itself. Native unit planning grouped selected Cargo dependency artifacts by package, so the package's bin-to-lib self edge leaked back onto the package lib unit.

## What Changes

- Keep selected dependency artifacts package-scoped for external dependencies, but normalize them per target before creating native target units.
- Drop same-package dependency artifacts from non-bin target units.
- Re-add the package lib artifact only for bin targets that actually need their same-package library.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/changes/rust-topology-self-package-lib-binding/**`, `cairn/specs/rust-package-planning/spec.md` after archive.
- **Testing**: focused native unit graph tests, combined topology regression tests, dirty/clean self-probes, Cairn validation/gates.
