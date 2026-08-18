## Why

The latest Mantle release evidence proves reproducibility for named artifacts, but the proof is still a single bounded release context. To strengthen the claim, Mantle needs a repeatability matrix that reruns the same release artifact builds across clean stores, scrubbed environments, cache modes, users, temp roots, and host classes, then records exactly which axes matched or blocked.

## What Changes

- Add a release repeatability matrix workflow that derives a deterministic run plan from a release bundle, policy, and declared matrix profile.
- Execute each matrix cell in isolated output/store roots with recorded ambient controls and cache/substitution settings.
- Emit a matrix report that preserves matched digest sets, mismatches, missing outputs, reused-store blockers, and non-claims.
- Feed eligible matrix evidence into global reproducibility evaluation without bypassing the existing global admission gate.

## Impact

- **Files**: release reproducibility core, release CLI, report schemas, tests, docs, Cairn verification-evidence spec delta.
- **Testing**: positive multi-axis matching run, negative forced mismatch/missing-output run, stale/reused-store blocker checks, Cairn validation/gates.
