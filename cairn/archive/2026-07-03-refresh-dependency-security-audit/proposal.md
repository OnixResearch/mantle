## Why

Mantle's dependency audit policy has known configuration and waiver details. As dependencies and advisories move, the repo needs a current audit pass that distinguishes fixed advisories, upstream-blocked risks, and policy/tooling failures. This keeps security status evidence current without hiding known vendor constraints.

## What Changes

- Run the current dependency/security audit rails using the repo's checked-in policy.
- Classify advisories and license findings into fixed, accepted-waiver, upstream-blocked, or action-required buckets.
- Update dependency pins, waivers, or documentation only when supported by evidence.
- Add a concise evidence transcript and avoid claiming a clean audit unless the command proves it.

## Impact

- **Files**: `deny.toml`, Cargo manifests/lockfile if updates are needed, dependency audit docs/evidence, Cairn verification-evidence spec delta.
- **Testing**: `cargo deny` or equivalent audit rail, lockfile consistency checks, negative policy-missing check if feasible, Cairn validation/gates.
