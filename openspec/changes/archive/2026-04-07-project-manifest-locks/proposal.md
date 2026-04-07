## Why

crunch has a solid build engine, but weak project-level input management. A
user who wants pinned external sources still has to hand-roll `seed.ncl`,
fetch helpers, update flows, and source rewrites. That is workable for small
examples, but it does not scale to real package sets or long-lived projects.

`../nixtamal/` already solves the workflow problem on the Nix side:
separate the human-edited manifest from the machine-edited lock, provide
refresh and stale detection, track mirrors and patches, and version the
format from the start. crunch should adopt that model, but not nixtamal's
KDL syntax or its Nix-prefetch implementation.

The project layer is a different concern from eval/build/store. It should not
be folded into the `crunch` binary crate or the build pipeline crates.

## What Changes

- Add a new workspace crate, `crunch-project`, for project manifests,
  lockfiles, stale detection, refresh, upgrade, and generated input files.
- Introduce a Nickel-native project manifest (`crunch-project.ncl`) and a
  machine-generated JSON lockfile (`crunch.lock`).
- Generate `.crunch/inputs.ncl` from the lock so package code can import
  resolved inputs without network activity during evaluation.
- Add project commands: `crunch init`, `crunch check`, `crunch show`,
  `crunch refresh`, `crunch list-stale`, and `crunch upgrade`.
- Add first-class mirrors and patch metadata to project inputs.
- Keep fetch execution in the existing fetch/build pipeline. The project layer
  records and resolves input metadata; it does not become a second build
  engine.

## Capabilities

### New Capabilities
- `project-management`: human-edited Nickel manifests, JSON lockfiles,
  refresh, stale detection, generated inputs, mirrors, patches, and upgrades

### Modified Capabilities
- `cli`: gains project-management commands in addition to `build` and `eval`
- `architecture`: adds a dedicated project crate instead of embedding this
  logic in the binary crate
- `package-authoring`: can import locked inputs from a generated file instead
  of hand-maintained source records

## Impact

- **Files**: new `crates/crunch-project/`; new project files
  `crunch-project.ncl`, `crunch.lock`, `.crunch/inputs.ncl`; CLI wiring in
  `src/main.rs`
- **APIs**: new project-layer types for manifests, locks, upgrades, and input
  materialization
- **Dependencies**: no Nix-prefetch or KDL dependency; keep the format and
  update logic crunch-specific
- **Testing**: unit tests for manifest/lock parsing and merge; integration
  tests for refresh, stale detection, upgrades, and generated input files

## Out of Scope

- Copying nixtamal's KDL format into crunch
- Eval-time fetching or Nix-dependent prefetch commands
- New VCS backends beyond crunch's existing fetch story
- TUI progress rendering
- GC generations and named roots
