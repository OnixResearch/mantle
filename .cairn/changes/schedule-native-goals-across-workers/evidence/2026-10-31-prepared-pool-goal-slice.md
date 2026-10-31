# Prepared pool goal envelope slice (2026-10-31)

Scope: the resolved-facts envelope the executor seam must carry, because a bare
`BuildRequest` cannot express pool placement requirements.

## Contract (`crunch_build::distributed`)

- `PreparedPoolGoal` pairs the concrete `snix_build::buildservice::BuildRequest`
  with the facts the Worker already resolved: realization key, system, required
  features, sandbox and network modes, semantic accelerator classes, placement
  key digest, and the requested `resource::Capacity`.
- `PreparedPoolGoal::new(PreparedPoolGoalFacts)` rejects an empty/oversized
  system, an empty sandbox or network mode, a non-BLAKE3 placement key, more
  than 32 combined capabilities, and an empty/oversized capability name.
- `PoolGoalSelector::select(goal, endpoint_ids)` is the selection contract for
  this envelope: a selector sees the resolved facts, never a bare request.

This is deliberately a *new* input to selection, not a re-derivation from
`BuildRequest`. The design's seam finding records why: the request has no
realization key, capability requirement, execution policy, or prepared capacity,
so inferring placement from it would fabricate facts.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-build --lib distributed::tests::` | 48 passed (adds the envelope and a facts-driven selector case) |
| strict Clippy `crunch-build`, `cargo fmt --check -p crunch-build` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

Fixtures reject each malformed fact class and show a selector choosing an
endpoint from `system` and `required_features` rather than from the request.

## Not claimed

- The Worker does not produce this envelope yet; choosing between extending the
  Worker-to-executor seam and selecting in the existing `ReadyDerivationGoal`
  path remains the open decision recorded in the design.
- The envelope carries no capacity observation and authorizes no dispatch.
