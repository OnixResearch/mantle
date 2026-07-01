# Current Blocker — Project input fetch policy

Date: 2026-06-30

## Question

Can `input-fetch-policy` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/input-fetch-policy/tasks.md` still has 8 unchecked tasks covering schema/defaults, preflight semantics, pure validation/classification, refresh/generated-input/build/source-bundle/offline-preflight integration, and positive/negative/CLI tests.
- The spec requires policy to distinguish generation material, build fetch actions, and imported/offline source state, and requires offline preflight to fail before sandbox or remote dispatch when source state is missing.
- `offline-source-bundle-manifest` remains active and blocked; its current blocker says imported source-state readiness and offline build preflight integration are not complete.
- Current project code has refresh/list-stale/generate surfaces, but no implemented fetch-policy data model or offline preflight enforcement tied to imported source state.

## Decision

Blocked. The policy cannot be drained until Mantle has the source-state/offline-preflight substrate needed to enforce `imported-source-required` and network-blocked decisions without overclaiming offline readiness.

## Owner

Mantle project/source transport owner after source-bundle import/readiness and offline build preflight foundations land.

## Next action

1. finish or split the source-bundle/offline-preflight foundation;
2. implement pure fetch-policy classification over manifest/lock/source-state facts;
3. thread policy through refresh, generated inputs, build planning, source-bundle planning, and remote dispatch preflight;
4. add CLI tests proving check/list-stale do not fetch by default and offline preflight blocks before sandbox execution.
