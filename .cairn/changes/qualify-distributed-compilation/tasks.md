# Tasks: Qualify distributed compilation

Proposed qualification only. Every runtime/implementation task is unchecked.
Structural gates and local loopback cannot complete independent-host tasks.

## Phase 1: Manifest and maintained runner

- [ ] [task:Q1][serial] Retain accepted frontend/pool contracts and define a typed Nickel/exported manifest for exact workload/tool/policy/input identities, expected outcomes, required cells, admitted independent hosts, credential FD references, resources and bounds. r[distributed_compilation.qualification_manifest]
- [ ] [task:Q2][after:Q1] Implement the owned-run-directory proof runner/report verifier with separate loopback/independent-host modes, preflight before effects, immutable attempt records and explicit unavailable outcomes; document its command/claim/non-claims in the operator proof guide. r[distributed_compilation.qualification_manifest] r[distributed_compilation.repeatable_evidence]
- [ ] [task:Q3][after:Q2] Bind actual compiler exec/lifetime evidence to current workers/attempts and one coordinator monotonic domain; establish overlap without wrapper-only timing or unsynchronized host-clock assumptions. r[distributed_compilation.multihost_execution]

## Phase 2: Positive workload matrix

- [ ] [task:Q4][after:Q3][parallel] Run real local/one-/two-worker cold Rust branching and proc-macro/build-script/generated-source graphs, then execute final programs with independent expected outcomes and retain full verified artifact barriers. r[distributed_compilation.multihost_execution]
- [ ] [task:Q5][after:Q3][parallel] Run real local/one-/two-worker C/C++ object/archive/link/install graphs and the mixed-language native-provider-to-Rust consumer; execute installed consumers and verify expected outputs and complete declared roles. r[distributed_compilation.multihost_execution]
- [ ] [task:Q6][after:Q4][after:Q5] Execute the required two-worker matrix on two operator-supplied independent Linux hosts over an admitted production transport with separate stores/keys; retain host/session provenance and actual compiler overlap, or report the exact missing prerequisite without substituting loopback. r[distributed_compilation.multihost_execution]

## Phase 3: Hostile and failure matrix

- [ ] [task:Q7][after:Q6][parallel] After genuine successes, inject real compiler/script rejection, stale objects/generated outputs, changed full keys/NAR/prefix/output/attestation and stale fences; inspect no false dependent execution or final PathInfo/action-result success. r[distributed_compilation.qualification_failures]
- [ ] [task:Q8][after:Q6][parallel] Exercise capacity saturation/shrink/unknown vectors, concurrent completions, worker death, disconnect/timeout/cancellation, attached waiters and transfer interruption/corruption; retain exact current attempt/lease ownership and observed teardown/recovery. r[distributed_compilation.qualification_failures]
- [ ] [task:Q9][after:Q6][parallel] Exercise positively reachable private-marker and rejected-binding controls, missing credentials/hosts/tools and unsupported backend/target rows; prove no unauthorized transfer/effects and no unavailable or infrastructure result promoted to passing coverage. r[distributed_compilation.qualification_manifest] r[distributed_compilation.qualification_failures]

## Phase 4: Fresh-state repetition and evidence

- [ ] [task:Q10][after:Q7][after:Q8][after:Q9] Repeat cold runs in new owned client/worker state and controlled warm/one-input-change runs; retain same-plan artifacts, independent program outcomes, actual compiler counts, transfer/reuse costs and every failed/unavailable attempt. r[distributed_compilation.repeatable_evidence]
- [ ] [task:Q11][after:Q10] Verify report/evidence linkage and exercise missing required cell, stale subject, altered raw bytes, false worker attribution and hidden-failure rejection; measure comparable repeated workloads without asserting a general speedup or changing denominators. r[distributed_compilation.repeatable_evidence]

## Phase 5: Qualification and lifecycle acceptance

- [ ] [task:Q12][after:Q11] Run the actual maintained proof command plus affected first-party formatting/Clippy/focused tests, no-std/Nickel checks and required repository gates; capture exact command/output evidence and keep missing heavy prerequisites nonpassing. r[distributed_compilation.repeatable_evidence]
- [ ] [task:Q13][after:Q12] Publish the bounded compiler/target/transport/backend support matrix, exact repeat commands, measured costs, unsupported cells, non-claims, owners and invalidation triggers in operator docs/examples and release notes; run pinned Cairn validation/all authoring gates and obtain acceptance before a separately authorized committed sync/archive. r[distributed_compilation.repeatable_evidence]
