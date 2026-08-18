## Context

Cargo caps lints for dependencies so upstream warnings do not fail downstream builds. Mantle direct rustc currently preserves dependency crate lint settings without Cargo's cap. `derive_builder_core@0.20.2` has a warning that becomes an error under its crate lint policy on the current toolchain.

## Decisions

### 1. Derive cap-lints from explicit source kind

**Choice:** Add a pure helper that checks the unit package's `SourceKind` in the source closure and returns true for `Registry` and `Git`, false for `Path` and unsupported/unknown sources.

**Rationale:** Source closure is already explicit receipt material. This avoids ambient Cargo state and keeps workspace/path crates strict.

### 2. Apply to host and target derivations

**Choice:** Append `--cap-lints allow` while deriving both native target units and native host units when their source kind is non-local.

**Rationale:** Registry dependencies can be normal libraries, proc macros, or build scripts. Cargo's dependency cap applies across those rustc invocations.

### 3. Do not broaden lint policy knobs

**Choice:** Do not model custom profile lint settings, `RUSTFLAGS`, or workspace lint tables in this change.

**Rationale:** The self-probe blocker is the dependency cap. Wider lint policy should be evidence-driven and separately specified.

## Risks / Trade-offs

- Git dependencies are treated like registry dependencies because they are non-local dependency sources. If Mantle later distinguishes path-overridden git sources, the source closure can carry that decision.
- Path/workspace crates remain uncapped, so local warnings can still fail when crate policy denies them.
