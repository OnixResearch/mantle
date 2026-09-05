# Per-stage publication contract

## Goal and evidence

Complete I3 without requiring later-stage artifacts to publish an earlier completed prefix. Each stage must publish its remeasured payloads and manifest before the next stage starts. An interrupted attempt must retain a usable prefix in a fresh staging directory.

Acceptance requires positive interruption-and-restore tests and negative stale, missing, conflicting, and modified-payload tests. Rollback must retain pre-existing payloads. Publication cannot relabel restored work as current execution. Promoted checkpoints and historical proof identities must remain unchanged.

## Scope and owner

Mantle owns stage meaning and resume policy in its existing pure modules. Its shell owns bounded observation, copying, persistence, and execution order. Existing content-addressed tree and no-replace checkpoint mechanisms remain the first reuse candidates. No new external capability is required merely to calculate a prefix.

The existing worktree and branch retain the repair. The failed `97f47ae2` attempt remains immutable. A later runtime cycle must bind a new committed source, binary, and source profile.

## Search budget

Use three correlated serial review passes: prefix shape, publication order, and restore ownership. No subagents run. Each pass permits four targeted source groups before implementation. Permit four focused repair rounds before the integration check. New counterexamples can justify a recorded, bounded extension.

Long runtime checks require named detached runs, immutable input bindings, explicit resource limits, and retained raw evidence. Do not duplicate a running attempt. Do not reuse V98 proof authority for the promoted cold proof.

Allowed outcomes are validated, blocked, exhausted, or user-decision-required. Tests, markers, and process exits cannot replace the required runtime observations.

## Approach registry

| Family | Mechanism | State | Next check |
| --- | --- | --- | --- |
| Prefix shape | Dev-only stage-prefix checkpoints with exact required payload sets | active | Identify the minimum payloads for each continuation |
| Independent stage objects | Resume manifests directly own stage objects | independent | Compare with existing remeasurement and restore validation |
| Publication order | Complete and publish a stage before its successor starts | active | Inspect provider and fixed-point stage boundaries |
| Restore ownership | Retain created-path ownership through admission | audit | Run existing ownership regressions before changes |

The promoted four-stage checkpoint contract must not weaken to admit a dev prefix. The final decision and evidence will update this registry.
