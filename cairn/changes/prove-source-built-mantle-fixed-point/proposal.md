## Why

Mantle has separate fixed-point evidence for the admitted full-source C provider and for a source-built Rust/native closure, but no proof constructs and consumes both within one authenticated lineage. The checked real-self-build parity evidence is also stale: it embeds `mantle-deterministic-proof-receipt-v1`, while the current verifier requires v2 content-bound rebuild authority.

A full-bootstrap fixed-point claim requires a fresh proof from source inputs, not composition of old output receipts. The proof must construct the native and Rust providers, build stage1 Mantle, use stage1 Mantle to build stage2 through the same Cargo-free path, and retain strict zero-fetch/zero-fallback evidence with matching BLAKE3 binaries.

The current six-stage plan names broad proof stages but does not provide one pre-execution list of every reachable action, tool, input authority, producer edge, locality decision, and limit. Stage-local audits remain necessary, but operators also need a cheap root-scoped report that fails before a long proof when the action list is incomplete.

## What Changes

- Add a full-bootstrap proof mode that consumes authenticated source bundles and constructs the StageX native provider plus full-source-bound Rust provider before Mantle compilation.
- Execute both Mantle stages through the receipt-bound Cargo-free topology and strict hermeticity policy.
- Run the first promoted V2 evidence build on Leviathan (`leviathan.cymric-daggertooth.ts.net`) as the sole execution host. Transfer the exact source tree and authenticated profile before launch. Use SSH, rsync, and pueue only for operator control, not proof-action dispatch.
- Emit a deterministic root-scoped action trust plan before construction. It lists every reachable action, fixed or producer-linked executable authority, input authority, output, execution locality, event bound, and resource limit.
- Reconcile the planned action list with protected-exec and build execution records. Reject unknown, missing, digest-mismatched, producerless, remote, cache-only, or over-limit execution.
- Emit current `mantle-deterministic-proof-receipt-v2` evidence linking source, lineage, toolchain closure, stage authority, action trust, outputs, protected execution, and non-claims.
- Preserve all failed and mismatched attempts without updating successful-proof aliases.

## Dependencies

- `bind-full-source-rust-provider`.
- `materialize-stagex-lineage-provider` (which transitively depends on both native parity changes).

## Impact

- **Files**: bootstrap functional core, self-build/Cargo-free orchestration, source-bundle profiles, deterministic proof/release evidence, parity inputs, proof scripts/tests, documentation, and lifecycle evidence.
- **Testing**: Add pure proof-plan and action-trust tests. Add negative authority, execution, fallback, and tamper cases. Record Leviathan preflight and source-transfer parity. Run the real proof, v2 receipt verification, release verification, and Cairn gates.

## Outcome

V98 completed the promoted proof on Leviathan from source commit
`af4b2d147d3b9fd0c216d3b1f6d11da1e043b810`. Stage1 and stage2 produced the
same BLAKE3 binary identity. The final root reconciliation matched all 1,914
actions and 478,870 local events with no findings. The independently rerun
bootstrap trust report returned `complete` with no blockers. Evidence is under
`evidence/v98-ptrace-request-abi-2026-08-31/`.
