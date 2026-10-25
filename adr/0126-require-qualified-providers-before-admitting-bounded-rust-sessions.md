# ADR 0126: Require qualified providers before admitting bounded Rust sessions

## Status

Proposed — execution admission blocked.

## Context

The open `bound-provisional-rust-execution`,
`pipeline-rust-interface-readiness`, and `reuse-warm-rust-workspaces` changes
require actual compiler/provider processes, not an application type definition
or a compiler-cache broker relabelled as retained compilation.

Fresh capability inspection on 2026-10-23 is retained in
[the interface-readiness admission record](../.cairn/changes/pipeline-rust-interface-readiness/evidence/authoring-validation.md#current-implementation-admission-inspection-2026-10-23).
The exact selected Nix compiler is Rust 1.96.0-nightly,
`c756124775121dea0e640652c5ee3c89e3dd0eb4`, from
`/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08`.
It rejects `-Zearly-metadata --print cfg`; the same closure's Cargo rejects
`-Zheadstart locate-project`. Version-only invocations do not process these
flags and are not capability evidence. The source-built provider plans select
upstream Rust 1.94.1 rather than a reviewed early-metadata patch identity.

The pinned [Headstart patch series](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/patches/README.md)
adds real compiler early-publication/swap behavior, but that pin's
[incremental SVH defect](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/ice-early-metadata-svh.md)
is explicitly unfixed. Its [readiness assessment](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/readiness.md)
also describes token-free resumption and uncapped resident compilers. No newer
provider is admitted merely because a newer repository revision might exist.

The pinned [rustd process](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/crates/rustd/src/main.rs)
retains a real database, but its `sync`/`patch` line protocol and mtime-based
change detection do not provide a bounded versioned content-bound revision
acknowledgment and stop/drain contract. Its toolchain is a separate
`nightly-2026-05-10` identity, not same-binary early-metadata off/on.

The inspected host exposes cgroup-v2 memory/task controllers, but this user's
current `session-7.scope` is root-owned, has no delegated subtree controllers,
and reads `max` for memory and tasks. That is not an admitted finite session
boundary. Sampling RSS or setting a per-process address-space limit would not
supply the aggregate resident-session enforcement required by these changes.

## Decision Drivers

- Require actual compiler/provider protocols and exact toolchain identities.
- Preserve completed-artifact authority and independently checked differential evidence.
- Enforce aggregate resident-session bounds rather than substituting RSS sampling.
- Keep unsupported runtime modes unavailable instead of supplying fake providers.

## Decision

1. Preserve the existing serial whole-unit execution and completed-artifact
   authority. Do not expose an enabled provisional, early-interface, or warm
   mode until an actual declared provider and enforcement mechanism satisfy
   their original requirements. No fake provider, restart-per-edit fallback,
   or unused future session port is introduced to suggest partial runtime
   delivery.
2. Admit compiler/Cargo bytes and source/patch provenance, sysroot/runtime
   closure, protocol and semantic arguments before dispatch. Early-interface
   qualification requires source-fixed ordinary incremental identity,
   unchanged-warm/edit/revert and full-metadata-swap evidence with identical
   binaries off/on. Disabling incremental, flushing caches, alternate metadata
   hashing, or replacing the compiler with rustd cannot satisfy that gate.
3. For a notification-only compiler, reserve producers first and retain each
   execution reservation while waiting. Both running and paused compilers
   remain resident and charged. Recycling on `wait-metadata` then accounting
   for `resume` after it occurs is not strict enforcement. A separate
   acknowledged pre-resume protocol would need separate review.
4. Require a real versioned resident-provider bridge before warm execution:
   quiescent content-bound update, exact partition/session/base/revision/request
   acknowledgment, coherent broken/still-broken/repaired lineage, bounded
   structured work observations, and stop/drain. Raw `sync` and mtime changes
   are not that bridge. Require the current revision's authoritative compiler
   check, provider/link success and complete output validation before execution.
5. Select and validate finite Nickel-authored policies against an enforceable
   session memory/task boundary before launch. Idle residents retain capacity.
   Cancellation, disconnect, malformed control and exhausted budgets stop
   admission and invalidate pending output; capacity remains owned until
   observed contained-tree teardown. An expired teardown deadline is an
   incomplete-teardown failure, never successful cleanup or a reusable slot.
6. Keep provisional outputs and bounded ordered diagnostics private. Confirm
   the exact producer-attempt closure before invoking any existing success,
   publication, artifact-consumption or script-execution hook. Keep complete
   source/compiler/options/environment/dependency identity in the action key;
   interface identity cannot substitute for it. A different successful producer
   attempt cannot retrospectively confirm an earlier dependency.

## Integration boundary

The pure core owns deterministic exact-attempt readiness, confirmation,
invalidation and policy decisions; the application owns their orchestration;
a qualified shell owns processes, private staging, live bounded control,
resource enforcement, clocks and teardown. No generic scheduler `Done` state
or PathInfo authority acquires a provisional meaning.

`execute_existing_units` currently finishes each unit before visiting its
consumers. Its `execute_miss` shell path calls
`finalize_successful_rust_unit_execution` in `src/rust_plan.rs`, which creates
success receipts and publishes local/shared cache results **before** the
application classifies the returned process observations. Therefore a future
real session must gate that finalizer itself, not only delay a returned
`Succeeded` observation. It must also gate
`NativeTopologyUnitExecutor::finish_unit`, which records final libraries/host
artifacts and can run build scripts. The existing final-output verification
and signed-candidate visibility order remain mandatory after confirmation.

The current wrapper V1 admits one complete output file and buffers compiler
streams until process completion. It is not a live compiler-control transport
or a resident multi-output transaction. Its ordinary fail-open cache policy
must not become a fallback from rejected experimental session admission.

## Consequences and acceptance boundary

The three feature changes remain open and unaccepted. This record adds no
execution implementation, provider import, default switch, measurement,
performance claim, or unsupported capability. No compiler workload, build or
test was run for this admission inspection. Existing ordinary full-metadata
and serial baselines remain independently usable but cannot qualify a blocked
lane or establish retained compiler work.

Enabling implementation requires a qualified compiler/provider closure, the
selected enforceable finite host boundary, and the original positive and
negative real-process matrix. In particular it must prove no provisional
cache/receipt visibility, no wrong-attempt confirmation, observed teardown
before release, and retained same-lineage repair for warm workspaces. A
source-fixed compiler and a qualified resident provider are different
prerequisites; delivering either does not silently satisfy the other.
