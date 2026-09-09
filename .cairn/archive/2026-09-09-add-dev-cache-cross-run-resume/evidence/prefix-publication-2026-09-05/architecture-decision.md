# Prefix publication decision

## Selected mechanism

Dev prefixes use their own schema and manifest namespace. They reuse the existing provider checkpoint identities, bounded tree observation, no-follow copying, and no-replace publication. Shared objects use separate content-kind namespaces and BLAKE3 names.

The first three provider boundaries require 1, 2, and 14 payloads. The fourth requires all 17. The native boundary includes its host tools and their evidence. It does not require the later Rust provider, toolchain closure, or Rust action evidence.

The existing promoted schema still requires four stages and all 17 payloads. Its identity context remains unchanged. A dev prefix cannot change its origin or schema to gain promoted admission.

The core resume plan selects a manifest. Its explicit checkpoint reference selects the provider prefix. The loader does not substitute an unrelated or more complete checkpoint.

## Publication order

The provider shell publishes after each completed boundary. Mantle stage 1 uses an application-owned publication port before stage 2 starts. A failed publication prevents continuation. Stage 2 publishes only after fixed-point validation succeeds.

An attempt-local manifest journal records successful publications. Final reporting reads that bounded journal instead of publishing every stage after final success. Restored stages do not create new publication records merely because they restored successfully.

## Restore and authority checks

Mutable host bindings require a payload copy owned by the current restore. Existing identical payloads cannot become relocation scratch. Partial native prefixes relocate host-tool manifests through the same helper as complete provider bindings.

Dev native commands already permit missing derivation events from store reuse. The shell now retains their exact aggregate observations. Restore replays those events against the retained plan and requires exact reconciliation equality. Promoted restore still requires complete reconciliation. Neither path turns missing events into fresh execution.

A read-only copy of the failed cold attempt's reconciliation records 88 planned and matched actions, with 568 matched events. It contains no missing actions or cache-only completions. This observation does not establish the later cached cycle.

## Alternatives

| Family | Decision | Reason |
| --- | --- | --- |
| Dev prefix checkpoints | Selected | Reuses existing identities and restore validation without requiring future artifacts |
| Independent stage object format | Rejected | Duplicates payload policy and semantic restore checks |
| Weaken the promoted checkpoint format | Rejected | Mixes dev reuse with promoted proof authority |
| Publish only after full success | Rejected | Cannot recover an interrupted first run |
| Use one global provider checkpoint during selection | Rejected | Can substitute a different content reference for the selected manifest |

## Audit scope

The review passes are serial and correlated. The native boundary exposed additional required facts: host-tool payloads, later-only toolchain closure, and cache-backed action observations. The review added a bounded pass over those existing helpers and their restore path.

Focused tests cover prefix shape, shared objects, mutation, symlinks, fresh-directory restoration after removal of the earlier attempt, and publication-before-continuation. Runtime confirmation and the separate promoted proof remain V2 and V3 work. The pending Nix and committed-source checks still gate the implementation commit.
