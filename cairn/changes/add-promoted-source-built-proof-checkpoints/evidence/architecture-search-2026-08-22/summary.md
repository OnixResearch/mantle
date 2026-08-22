# Promoted checkpoint architecture search

## Goal

Reuse completed provider work in later promoted proof attempts without weakening source admission or claiming current execution.

Completion requires an immutable checkpoint, stage-specific authority, complete prior execution evidence, fresh-root restore, payload remeasurement, and explicit restored-stage receipt fields.

False completion includes dev-cache adoption, path-only trust, digest-only trust, same-directory resume, a cache hit without execution evidence, or a final receipt that labels restored work as current execution.

## Search budget

The search used four correlated local lenses and no subagents:

1. existing source-built dev cache and resume code;
2. shared action-result and Mantle store contracts;
3. active and archived Cairn checkpoint work; and
4. current six-stage plan and v2 receipt construction.

## Approach registry

### Dev cache promotion

- Mechanism: allow `--dev-provider-cache` in promoted proofs.
- Artifact: `src/source_built_fixed_point_dev_cache.rs`.
- Evidence: exact source and five policy digests already gate provider adoption.
- Gap: stronger than a raw cache hit, but weaker than the goal.
- Blocker: dev entries omit complete StageX payload and promoted execution-evidence composition.
- State: rejected as direct reuse; retained as implementation reference.

### Shared action-result reuse

- Mechanism: discover every prior action through the action-result store.
- Artifact: ADR 0024 and `crunch-action-result-core`.
- Evidence: clean-client reuse already separates lookup, content, admission, and execution.
- Gap: valid but much wider than the first checkpoint need.
- Blocker: StageX transition is not one ordinary Mantle derivation, and thousands of action records do not restore the required execution tree.
- State: retained as semantic precedent, not the first shell route.

### Same-attempt resume

- Mechanism: trust stage markers in the failed staging directory.
- Artifact: `--dev-resume` and `StageCompletionMarker`.
- Evidence: marker validation exists for StageX.
- Gap: simpler but weaker than the goal.
- Blocker: mutable attempt state, no fresh-root restore, and no promoted receipt composition.
- State: rejected.

### Stage-specific promoted checkpoint

- Mechanism: publish one immutable provider-closure checkpoint with four stage records and complete payload evidence.
- Artifact: `src/source_built_fixed_point_checkpoint.rs` and this change package.
- Evidence: the pure core derives authority only from the source, policy, resource, and predecessor outputs used by each stage.
- Gap: matches the requested provider-reuse goal.
- Blocker: shell publication, restore, receipt integration, and runtime evidence remain.
- State: selected and active.

## Adversarial audit

The selected key excludes Mantle source and vendor inputs only because they first enter stage1. Tests prove that a Mantle-only change preserves lookup.

The same tests prove that Rust-provider source or protected-execution policy changes alter lookup. Promoted admission rejects dev origin, payload mutation, forbidden events, and wrong semantic provider identity.

The checkpoint will not use its lookup key as authority. Restore remains blocked until every payload and stage record revalidates.

## Decision

Implement the stage-specific promoted checkpoint. Keep shared action-result admission rules and dev cache code as reusable mechanisms, but preserve a separate promoted schema and claim boundary.

## Owner and next action

The source-built fixed-point shell owns checkpoint orchestration. The checkpoint core owns deterministic admission. The next action is bounded no-follow payload publication and fresh-root restore.
