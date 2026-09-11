# Tasks: Add machine-wide compile slots

All implementation and acceptance tasks remain open. Proposal creation is not
producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current per-build scheduling bounds, the daemon surface, and a contention observation from concurrent bootstrap builds. r[mantle.compile_slots.slot_authority]
- [ ] [serial] T1.2 Define the versioned slot policy: totals, per-build reservation bounds, queue limits, fairness order, and receipt schema as a typed Nickel export. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.bounded_fairness]
- [ ] [serial] T1.3 Record the same-daemon and advisory-capable decisions in an ADR. r[mantle.compile_slots.degrade_without_authority]

## Phase 2: Core authority

- [ ] [serial] T2.1 Implement pure grant policy evaluation: admission, fairness ordering, queue bounds, and typed rejections over in-memory request state. r[mantle.compile_slots.bounded_fairness]
- [ ] [serial] T2.2 Serve the authority from the existing daemon with grant and release receipts. r[mantle.compile_slots.slot_authority]
- [ ] [parallel] T2.3 Add positive fixtures: bound respected under concurrent builds, bounded grant wait, fairness ordering under a monopolizing build. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.bounded_fairness]
- [ ] [parallel] T2.4 Add negative fixtures: queue-limit rejection, absent authority degradation, authority death mid-build with safe release. r[mantle.compile_slots.degrade_without_authority]

## Phase 3: Driver integration

- [ ] [serial] T3.1 Integrate slot request and release into the compile driver seam from `extend-compile-cache-to-cc`, with immediate compilation when no authority answers. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.degrade_without_authority]
- [ ] [serial] T3.2 Prove output identity with slots on and off on a representative build. r[mantle.compile_slots.no_output_influence]
- [ ] [serial] T3.3 Keep scheduling receipts outside the derivation graph and rejected as build evidence in strict lanes. r[mantle.compile_slots.no_output_influence]

## Phase 4: Verification

- [ ] [serial] T4.1 Run a contention scenario with concurrent bootstrap builds and record grant receipts, observed concurrency, and fairness outcomes. r[mantle.compile_slots.bounded_fairness]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.compile_slots.slot_authority]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.compile_slots.no_output_influence]
