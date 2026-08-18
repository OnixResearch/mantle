## Why

Mantle's strongest proof claims rely on slow self-build and Cargo-free fixed-point commands. After topology and guard changes, stale evidence is worse than no evidence because it can hide a regression or overstate the current proof frontier. This change refreshes the expensive proof evidence and records the exact success or blocker state.

## What Changes

- Run the current expensive proof path with the documented environment.
- Store a concise evidence summary that names commands, output bundle paths, digests, and final verdict.
- If the proof blocks, record the current blocker class and next action without claiming success.
- Update docs/status wording that references the proof so it points to current evidence.

## Impact

- **Files**: proof evidence transcript, README/proof docs if stale, possibly proof-runner diagnostics if the run exposes a small fix, Cairn verification-evidence spec delta.
- **Testing**: expensive proof command or intentionally bounded blocker rerun, evidence-bundle inspection, stale/missing evidence negative check, Cairn validation/gates.
