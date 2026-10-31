# Tasks: Schedule native goals across remote workers

Planning only. Every unchecked task is an implementation or acceptance
obligation; authoring validation cannot satisfy it. Dependency archive state
is enforced by `metadata.json`, not waived by these structural gates.

## Phase 1: Contracts and baseline

- [ ] [task:S1][serial] Record current local, single-builder and dynamic-plan behavior with exact source/binary/config identities; admit the prerequisite remote ports and endpoint-binding contracts. r[distributed_compilation.compatibility_evidence]
- [ ] [task:S2][after:S1] Extend the existing Nickel pool schema and Rust admission for proposed pool flags, per-endpoint FD references, current bindings, full output keys, resource profiles and bounded session policy; migrate all affected configuration/report consumers. r[distributed_compilation.pool_selection]
- [ ] [task:S3][after:S2] Define the Mantle-owned prepared execution/result envelope carrying resolved derivation/action identity, declared closure and expected outputs; retain pure placement/failure decisions in the existing remote core and run references before changing exported APIs. r[distributed_compilation.concrete_payload]

## Phase 2: Production cutover

- [ ] [task:S4][after:S3] Implement the native-pool execution adapter through the remote application ports and canonical store capabilities; preserve binding, credential, transfer, attestation and full-key verification rather than signing unverified results locally. r[distributed_compilation.output_admission]
- [ ] [task:S5][after:S4] Connect local/native-pool selection to the ordinary Worker for static and late dynamic-plan goals; remove the local-as-remote composition and obsolete native serial dispatch loops, keeping portable/single-builder public contracts explicit. r[distributed_compilation.goal_dispatch] r[distributed_compilation.compatibility_evidence]
- [ ] [task:S6][after:S5] Persist/reconcile current fenced reservations under concurrent dispatch/completion; enforce global jobs, worker concurrency, explicit resource quantities, session/queue/transfer bounds and no lock across network I/O. r[distributed_compilation.resource_ownership]
- [ ] [task:S7][after:S6] Integrate last-owner cancellation, owned remote teardown, stale-result refusal and existing bounded retries; allow only admitted pre-dispatch fallback and preserve compiler/trust failure distinctions. r[distributed_compilation.terminal_behavior]
- [ ] [task:S8][after:S7] Finish verified outputs through the normal goal/store path, re-admit returned producer plan/source outputs, and add bounded per-goal placement/attempt/trust observations without changing ordinary JSON behavior. r[distributed_compilation.output_admission] r[distributed_compilation.compatibility_evidence]

## Phase 3: Behavioral acceptance

- [ ] [task:S9][after:S8][parallel] Run a real two-worker diamond and returned dynamic-plan graph with barriers, transitive inputs and a shared dependency; inspect overlap, exact compile/executor counts, final output bytes and one waiter notification per goal. r[distributed_compilation.goal_dispatch] r[distributed_compilation.concrete_payload]
- [ ] [task:S10][after:S8][parallel] Exercise concurrent coordinators/completions, saturated/unknown/shrinking resources, bounded queues, worker death, cancellation and stale fences; prove current lease ownership and no live owned descendants or duplicate accepted result. r[distributed_compilation.resource_ownership] r[distributed_compilation.terminal_behavior]
- [ ] [task:S11][after:S8][parallel] Pair valid full-key/NAR/attestation imports with same-name-different-key, tampered bytes, missing/extra output, revoked binding, private input and malformed credential cases; inspect absence of inadmissible store publication and network effects. r[distributed_compilation.pool_selection] r[distributed_compilation.output_admission]
- [ ] [task:S12][after:S9][after:S10][after:S11] Run actual local, one-worker pool, existing single-builder, signed-cache-hit and plan-only CLI scenarios; reject unsupported modes/backends before mutation and retain exact report compatibility evidence. r[distributed_compilation.compatibility_evidence]

## Phase 4: Verification and delivery

- [ ] [task:S13][after:S12] Run focused remote/core/application/build/pipeline/CLI regressions, changed-package formatting, strict first-party Clippy, no-std boundaries, applicable Nickel checks and repository-required quality gates; retain exact commands and raw results. r[distributed_compilation.compatibility_evidence]
- [ ] [task:S14][after:S13] Document pool configuration, credential use counts, admitted transports/backends, resource limits, fallback and local-loopback non-claims; update operator catalog/generated command docs through their owner and add a release note. r[distributed_compilation.pool_selection] r[distributed_compilation.compatibility_evidence]
- [ ] [task:S15][after:S14] Run pinned Cairn validation and proposal/design/tasks gates, retain implementation evidence and obtain acceptance; commit implementation before any separately authorized spec sync/archive. r[distributed_compilation.compatibility_evidence]
