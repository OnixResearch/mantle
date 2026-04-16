# Add operator diagnostics

## Why

Crunch still asks operators to infer too much from scattered logs and ad hoc
commands. Missing prerequisites, store-permission problems, unexpected fallback,
and failing derivations are all diagnosable today, but not from one clear,
supported diagnostics surface.

The quickest quality-of-life gain is to add a small operator-facing diagnostic
surface: preflight checks, a no-build execution plan preview, and structured
failure reports that point at the failing root and the saved log.

## What Changes

- add `crunch doctor` for environment and runtime preflight checks
- add a no-build execution-plan preview for `crunch build`
- add typed failure diagnostics shared by human and JSON output
- document the intended troubleshooting workflow

## Capabilities

### New Capabilities
- `doctor-preflight`: verify the current host can run the selected crunch
  workflow before spending time on a failing build
- `build-plan-preview`: show what crunch expects to build, substitute, or reuse
  without executing the build
- `typed-failure-diagnostics`: report failing root, phase, error class, and log
  path in a structured way

## Impact

- **Files**: CLI parsing and dispatch, build reporting, troubleshooting docs,
  possibly shared report types in pipeline-facing crates
- **APIs**: new diagnostic report types and planning entry points
- **Dependencies**: none required
- **Testing**: doctor success or failure cases, plan-mode no-mutate behavior,
  JSON failure envelope tests, and human-readable error snapshots

## Non-Goals

- graphical dashboards
- automatic remediation of failed preflight checks
- a full dependency-query surface such as `why-depends` in this change
- changing build semantics beyond diagnostics and planning
