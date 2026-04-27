# Tasks: Explore compiled evaluation backends

## Phase 1: Scope and specification

- [x] I1 Update proposal and delta spec to authorize the feature-gated prototype while preserving default interpreter/runtime constraints. [covers=r[compiled-eval.backend-boundary],r[compiled-eval.profiling-gate],r[compiled-eval.cranelift-prototype-subset]]
  - Evidence: `proposal.md` and `specs/compiled-eval-backends/spec.md` now name the opt-in `cranelift-proto` scope, concrete benchmark evidence paths, and non-shipping default runtime behavior.
- [x] I2 Update design to align current prototype evidence with the proposal non-goals and dependency policy. [covers=r[compiled-eval.benchmark-guardrail],r[compiled-eval.private-backend-seam],r[compiled-eval.cranelift-first],r[compiled-eval.future-semantics]]
  - Evidence: `design.md` decisions 1-8 cover the `crunch-eval` boundary, benchmark guardrail, private seam, Cranelift-first prototype, future semantics, and tabled status.

## Phase 2: Implementation status reconciliation

- [x] I3 Confirm the private `crunch-eval` backend seam remains the only integration point for compiled-eval experiments. [covers=r[compiled-eval.backend-boundary],r[compiled-eval.private-backend-seam]]
  - Evidence: `crates/crunch-eval/src/backend.rs` owns `EvalBackend`, `EvalRequest`, and `NickelBackend`; public callers still use existing helpers in `crates/crunch-eval/src/lib.rs`.
- [x] I4 Confirm the Cranelift prototype remains opt-in, subset-only, and non-shipping by default. [covers=r[compiled-eval.cranelift-first],r[compiled-eval.cranelift-prototype-subset]]
  - Evidence: `crates/crunch-eval/Cargo.toml` gates Cranelift dependencies behind `cranelift-proto`; `crates/crunch-eval/src/cranelift_proto.rs` handles only flat derivation literals.
- [x] I5 Reassess LLVM and broader compiled-eval expansion after lazy-root evaluation. [covers=r[compiled-eval.profiling-gate],r[compiled-eval.cranelift-first],r[compiled-eval.future-semantics]]
  - Evidence: archived `openspec/changes/archive/2026-04-18-lazy-root-evaluation/design.md` decision 5 tables further compiled-eval work until lazy metrics justify reopening it; no LLVM-specific requirement has been observed.

## Phase 3: Verification

- [x] V1 Record interpreter/prototype test evidence and current fresh-run blocker. [covers=r[compiled-eval.backend-boundary],r[compiled-eval.private-backend-seam],r[compiled-eval.cranelift-prototype-subset]] [evidence=openspec/changes/explore-compiled-eval-backends/evidence/V1-eval-tests.md] ✅ 14m (started: 2026-04-27T12:31Z → completed: 2026-04-27T12:45Z)
  - Evidence: fresh pueue task 17 was blocked by long Nickel compilation and killed after 9m11s; `V1-eval-tests.md` records the blocker and retains prior checked-in test-result evidence (`41 passed`, `35 passed`, `4 passed`).
- [x] V2 Record benchmark compare evidence and current fresh-rerun blocker. [covers=r[compiled-eval.profiling-gate],r[compiled-eval.benchmark-guardrail]] [evidence=openspec/changes/explore-compiled-eval-backends/evidence/V2-benchmark-compare.md] ✅ 14m (started: 2026-04-27T12:31Z → completed: 2026-04-27T12:45Z)
  - Evidence: fresh benchmark rerun was blocked before reaching benchmark commands; retained compare evidence shows guardrail drift (`build-graph-package-set.build_graph_wall_ns` +26.15%), so the prototype remains non-shipping and tabled.
- [x] V3 Run OpenSpec validation plus proposal/design gates before task-gate closeout. [covers=r[compiled-eval.backend-boundary],r[compiled-eval.profiling-gate],r[compiled-eval.benchmark-guardrail],r[compiled-eval.cranelift-first],r[compiled-eval.private-backend-seam],r[compiled-eval.cranelift-prototype-subset],r[compiled-eval.future-semantics]] [evidence=openspec/changes/explore-compiled-eval-backends/evidence/V3-openspec-gates.md] ✅ 17m (started: 2026-04-27T12:31Z → completed: 2026-04-27T12:48Z)
  - Evidence: `openspec validate explore-compiled-eval-backends` passed; proposal and design gates returned `VERDICT: PASS`; this tasks gate is the external closeout check after task closure.
- [x] V4 Record unsupported-shape coverage for the Cranelift prototype subset. [covers=r[compiled-eval.cranelift-prototype-subset.unsupported]] [evidence=openspec/changes/explore-compiled-eval-backends/evidence/V4-unsupported-subset.md] ✅ 1m (started: 2026-04-27T12:49Z → completed: 2026-04-27T12:50Z)
  - Evidence: `backend.rs` rejects import paths, `cranelift_proto.rs` accepts only the six documented flat-literal fields, and `prototype_rejects_nested_inputs` exercises the unsupported-field path; broader Nickel semantics stay future-scoped under `r[compiled-eval.future-semantics]`.

