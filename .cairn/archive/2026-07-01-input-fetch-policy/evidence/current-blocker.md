# Current Blocker — Project input fetch policy

Date: 2026-07-01

## Question

Can `input-fetch-policy` be honestly drained from the current tree?

## Inspected evidence

- `offline-source-bundle-manifest` is archived, so the imported source-state/offline-preflight substrate is available for this change to build on.
- `crates/crunch-project-core/src/fetch_policy.rs` defines the pure fetch-policy model, deterministic requirement classification, generated-input modes, compatibility diagnostics, source-state facts, source-identity matching, and no-fetch lock lowering.
- `mantle refresh`/`list-stale`, generated `.mantle/inputs.ncl`, `lib/fetch.ncl`, and source-bundle record planning carry fetch-policy data without silently widening network behavior.
- Focused validation transcript: `cairn/archive/2026-07-01-input-fetch-policy/evidence/implementation-validation-2026-07-01.md`.
- Sync and archive both executed, and post-archive `cairn validate` passed.

## Decision

Completed. The previous source-bundle/offline-preflight prerequisite was complete enough for `input-fetch-policy`; this change is now implemented, synced into accepted specs, archived, and validated.

## Owner

Mantle project/source transport owner.

## Next action

Drain dependent active changes (`forge-agnostic-vcs-inputs`, `project-freshness-probes`, `project-input-retention-roots`, and `project-input-trust-policy`) in follow-up work.
