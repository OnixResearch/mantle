# Tasks

All implementation and acceptance tasks remain open. Proposal creation is not producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Preserve unrelated changes and active proofs. Record focused core, shell, and Nix baselines plus the retained same-derivation failure. r[mantle.spacewasm_stable_evidence.rebuild]
  - Partial (2026-09-10): worktree `drain/spacewasm-evidence-20260910` from `origin/main` `3f0231a46`; focused core baseline recorded below. Nix baseline and full gate baselines still open.
- [x] [serial] T1.2 Define the versioned stable-fact contract, exact expected inventories, admissible presentation differences, named bounds, and typed Nickel export. Record the structured-format versus closed-text-grammar decision in an ADR. r[mantle.spacewasm_stable_evidence.contract]
  - Evidence: `crates/crunch-spacewasm-core/src/stable_report.rs` defines `mantle-spacewasm-stable-report-v1` with named bounds (`MAX_STABLE_TESTS`, `MAX_HARNESS_LINES`, `MAX_TEST_NAME_BYTES`) and canonical ordering; ADR 0079 selects the structured libtest JSON harness grammar over a text adapter. Typed Nickel export remains open pending producer integration.
- [ ] [serial] T1.3 Define the raw capture, run archive, retention authority, retrieval, and completeness contract. Reject identity cycles and nondeterministic secondary Nix outputs. r[mantle.spacewasm_stable_evidence.diagnostics]
- [ ] [serial] T1.4 Define reviewed delta modifications for any changed accepted materialization semantics. Define version compatibility, required member sets, migration, and unchanged historical verification. r[mantle.spacewasm_stable_evidence.handoff]
- [ ] [serial] T1.5 Review existing core helpers and compatible published capture/publication components. Record reuse decisions, exact pins for admitted dependencies, and visible composition roots. r[mantle.spacewasm_stable_evidence.boundary]
  - Partial: implementation reuses `crunch-spacewasm-core` digest, diagnostic, and result types only; no new dependency admitted. Full reuse review still open.

## Phase 2: Core and shell

- [x] [serial] T2.1 Implement pure bounded report admission, inventory comparison, canonical ordering, and BLAKE3 identity in the existing core. r[mantle.spacewasm_stable_evidence.contract] r[mantle.spacewasm_stable_evidence.boundary]
  - Evidence: `stable_report.rs` `parse_libtest_events` + `admit_stable_report` in `crunch-spacewasm-core`; identity excludes durations, order, and presentation; pure `no_std` core, no new dependencies.
- [x] [parallel] T2.2 Add positive equivalent-presentation fixtures and negative changed-test, changed-outcome, changed-command, feature, source, and executable fixtures. r[mantle.spacewasm_stable_evidence.contract]
  - Evidence: `tests/stable_report_fixtures.rs` equivalent-presentation identity equality, changed outcome and changed inventory identity changes; command/feature/source/executable identity edges are carried through the command identity field and profile binding and get producer-side integration in T2.5.
- [x] [parallel] T2.3 Add missing, duplicate, filtered, malformed, unknown-format, invalid-encoding, overflow, truncation, contradictory-summary, and unsupported-status controls. r[mantle.spacewasm_stable_evidence.denial]
  - Evidence: fixture controls for duplicates, malformed/unknown grammar, truncation, contradictory summary, empty captures, empty suite/command, missing expected tests, and over-bound names; 11 fixture tests plus 8 focused core tests pass (`test result: ok. 11 passed`); invalid-encoding (non-UTF-8) rejection lands with the shell capture boundary in T2.4.
- [ ] [serial] T2.4 Implement bounded execution and exact raw capture in the shell. Add nonzero-exit, signal, cancellation, timeout, unavailable-tool, capture-loss, permission, and retention-failure tests. r[mantle.spacewasm_stable_evidence.diagnostics] r[mantle.spacewasm_stable_evidence.denial]
- [ ] [serial] T2.5 Integrate the new contract into both upstream test producers and the complete bundle graph. Preserve selected tests, features, limits, roles, and required failure facts. r[mantle.spacewasm_stable_evidence.boundary] r[mantle.spacewasm_stable_evidence.handoff]

## Phase 3: Repeatability and independent verification

- [ ] [serial] T3.1 Add a repository-owned repeatability gate that forces fresh execution of every report-producing dependency in separate scratch roots. Record source and derivation identities, cache boundaries, raw captures, and complete stable-member comparisons. r[mantle.spacewasm_stable_evidence.rebuild]
- [ ] [parallel] T3.2 Add negative controls for reused report outputs, changed outcomes, missing members, bad parent edges, wrong schema or pin, tampering, and overclaims. r[mantle.spacewasm_stable_evidence.rebuild] r[mantle.spacewasm_stable_evidence.handoff]
- [ ] [serial] T3.3 Run focused core and shell tests before and after changes, core Wasm checks, strict Clippy, pinned Octet deny-all, Nickel freshness, independent bundle verification, and relevant Nix checks. Preserve exact blockers rather than suppressing checks. r[mantle.spacewasm_stable_evidence.boundary] r[mantle.spacewasm_stable_evidence.rebuild]
- [ ] [serial] T3.4 Run the complete frozen Nix acceptance check and the fresh-producer repeatability gate. Retain all commands, exit results, manifests, member comparisons, raw evidence, and bounded non-claims. r[mantle.spacewasm_stable_evidence.rebuild]

## Phase 4: Publication and consumer handoff

- [ ] [serial] T4.1 Publish the verified producer candidate as an immutable revision with contracts, migration notes, repeatability evidence, and positive and negative consumer fixtures. r[mantle.spacewasm_stable_evidence.handoff]
- [ ] [serial] T4.2 Obtain ChaosControl-owned evidence from a frozen consumer candidate with the new exact producer pin. Require the differential check and all manifest/member denial controls without consumer normalization. r[mantle.spacewasm_stable_evidence.handoff]
- [ ] [serial] T4.3 Resolve or record the existing repository-policy registry blocker through its owning change. Run native proposal, design, tasks, validation, and completion gates under the repository policy. r[mantle.spacewasm_stable_evidence.handoff]
- [ ] [serial] T4.4 Sync accepted specs and archive only after producer and consumer evidence passes. Integrate through the isolated branch with fresh upstream comparison and retain the completion evidence. r[mantle.spacewasm_stable_evidence.handoff]
