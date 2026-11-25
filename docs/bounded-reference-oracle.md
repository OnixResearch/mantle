# Bounded Reference Oracle and Differential Testing (`#40`)

This document describes Mantle's bounded reference evaluator and differential property suite in `crates/crunch-build/src/reference_oracle/` (implementing Issue `#40` and its Comment 1 Nickel composition fixtures under `mantle-scheduler-rebuilder-contract-v1`).

---

## 1. Scope and Contract

The differential oracle compares Mantle's production frontend conversion, goal scheduling, selected-output demand propagation (`#35`), cached-root preflight (`#41`), CA input resolution, shared action-result admission, dynamic-plan registration, and watch retraction (`Worker` + `Builder` + `crunch_glue` + `crunch_eval`) against an intentionally simple, independent reference model (`evaluator::evaluate_reference`).

### 1.1 Bounded Task Language (`task_language.rs`)
- **Explicit finite graphs (`TaskGraph`)**: bounded by `GraphBounds` (`max_tasks`, `max_edges`, `max_depth`, `max_dynamic_units`, `max_eval_steps`).
- **Explicit inputs & outputs**: static sources (`SourceSpec`), multi-output tasks (`["out", "dev", "lib"]`), `InputAddressed` and `ContentAddressed` addressing modes, selected-output dependencies (`TaskDep::SelectedOutput`), and Stage 2 dynamic plan producers and late-bound consumers (`TaskDep::PlanOutput`).
- **Independent evaluator (`evaluator.rs`)**: uses strictly iterative, bounded loops (`no_recursion`, `unbounded_loop` clean) and shares zero code or memoization tables with `crunch_glue::convert`, `ConversionCache`, `DerivationRegistry`, `Worker`, `Builder`, `crunch-action-result-core`, or `nix_compat::derivation`.

### 1.2 Explicit Three-Way Classification
`evaluate_reference` classifies every input into one of three distinct outcomes:
1. `ReferenceResult::Evaluated(ReferenceEvaluation)` — valid bounded DAG evaluated to completion, including expected output prefixes, runtime reference targets, `#35` transitive output demands, `#41` preflight-eligible roots, and Stage 2 dynamic plan branch outputs.
2. `ReferenceResult::Rejected(ReferenceRejection)` — explicit rejection of cycles, bound violations (`max_tasks`, `max_edges`, `max_depth`, `max_dynamic_units`), unknown/empty output selections, or invalid dynamic-plan bindings.
3. `ReferenceResult::Exhausted(ReferenceExhaustion)` — step-budget exhaustion (`steps >= max_eval_steps`). **Exhaustion is never coerced into rejection or pass**; differential tests explicitly fail if `Exhausted` is ever returned within test bounds.

---

## 2. Separate Cost Accounting

The instrumented harness (`test_support.rs` and `reference_oracle/harness.rs`) measures three separate operational cost classes and never conflates metadata probes with executor invocations or content transfers:

1. **Executor calls** (`DeterministicFakeBuildService::call_count`): invocations of `BuildService::do_build` and `BuildService::do_build_cancellable`.
2. **Metadata reads** (`CountingPathInfoService::{total_get_count, put_count, list_count}` and `CountingActionResultStore::{lookup_calls, publish_calls}`): `PathInfo` and `ActionResultStore` probes.
3. **Content transfers / store reads** (`CountingBlobService` `{has, open_read, open_write, chunks}` and `CountingDirectoryService` `{get, put, get_recursive}`).

This separation verifies `#41` preflight (`0` executor calls and `0` build-only ancestor `PathInfo` reads on warm preflight hits) and `#35` selected-output demand (`0` probes on unselected, unreferenced sibling outputs).

---

## 3. Covered Differential Scenarios, Mutation Checks, and Nickel Fixtures

- **All 8 `#40` scenarios**:
  1. Same `(name, builder, system)` with differing recipes (`#38`) in both root arrival orders;
  2. Diamond DAGs and duplicate/late root requests;
  3. Changed CA producer with unchanged realized CA content -> downstream reuse (`#40` early cutoff) plus negative control when realized content changes;
  4. Fresh vs cached dynamic-plan producer -> identical admitted units/bindings and strict consumer wait invariant (`GoalState::Waiting` while generated dynamic root is non-terminal);
  5. Missing intermediate objects or incomplete output blobs/closures -> reconstruction or explicit failure, never false `cached: true`;
  6. Tampered `PathInfo` signatures and conflicting valid `SignedActionResultRecord` candidates across candidate-order permutations;
  7. Watch retraction/replacement (`Worker::run_watch`) with barrier-gated late `Ok(BuildResult)` completion -> stale work discarded, zero `PathInfo` leakage;
  8. Invalid cycles, unknown outputs, and resource/step bounds -> explicit rejection (`Error` / `WatchWorkerEvent::Rejected`).
- **3 explicit mutation checks**:
  - `(a)` Coarse recipe key (`name:builder:system` memoizer);
  - `(b)` Dropped dependency wait (`worker::tests::run_with_dropped_wait_mutation` strictly inside `#[cfg(test)]`);
  - `(c)` Skipped admission condition (`trust_unsigned = true` and bypassed `plan_strong_reuse` conflict check).
- **Nickel composition fixtures (`#40` Comment 1, `nickel_fixtures.rs`)**:
  - Equivalence across inline records, imported constructor modules, composed higher-order combinators, `examples/diamond-dependency.ncl`, `EvaluationSession`, and `mantle.select`;
  - Store-prefix sensitivity (`drv_path` changes, `hdm` preserved);
  - Constructor elimination, order-permutation invariance, and distinct `provenance` preservation;
  - Boundary-attributed negative tests (`crunch_eval` vs `crunch_glue` vs `crunch-build`).

---

## 4. Evidence-Tier Distinction (`#40` vs Kani vs `#14`)

Mantle separates three complementary verification tiers and never treats one as a substitute for another:

| Tier | Mechanism | Where | What It Proves | What It Does Not Prove |
|---|---|---|---|---|
| **1. Differential Property Testing (`#40`)** | Bounded `proptest` (`ChaCha` RNG with pinned `RngSeed::Fixed(...)` seeds; overridable via `PROPTEST_CASES` / `PROPTEST_RNG_SEED`) + `DeterministicFakeBuildService` + mutation checks | `crates/crunch-build/src/reference_oracle/` | Runtime equivalence of conversion, scheduling, admission, dynamic plans, and watch retraction over finite bounded DAGs and mocked store/executor services | Unbounded graph sizes, arbitrary real sandbox execution, or universal absence of bugs outside tested bounds/strategies |
| **2. Bounded Model Checking (Kani)** | Exhaustive symbolic verification (`#[cfg(kani)]`) + release evidence binding | `crates/crunch-build/src/scheduling.rs`, `docs/kani-release-evidence.md` | Exhaustive symbolic invariants (comparator antisymmetry, transitivity, bounded epoch advance) on pure scheduling functions within Kani unwind bounds | End-to-end async `Worker` / `Builder` / store / sandbox behavior |
| **3. External Proof Ownership (`#14`, ADR 0034)** | Octet / Trellis / Aeneas / Verus proof profiles & opaque receipts | `adr/0034-*`, `crates/crunch-attestation/` | Cryptographic binding of externally checked theorem receipts, model/tool versions, and explicit assumptions | Mantle itself is not a theorem prover and performs no interactive proof checking |

---

## 5. Assumptions and Explicit Non-Claims

- **Deterministic mocked executor**: `DeterministicFakeBuildService` models recipe/input-deterministic execution; it does **not** prove arbitrary shell command correctness, compiler determinism, or Linux namespace/sandbox kernel isolation.
- **Visiting-identity & CA sibling-ref scope**: `crunch_glue::convert` treats repeated `(name, builder, system)` along a single active dependency chain as `CircularDependency` (while allowing colliding `(name, builder, system)` across distinct sibling/root recipes, tested in Scenario 1 and `proptest`); and multi-output CA derivations resolve self/sibling placeholders via `ca_plan` markers rather than `PathInfo.references` (CA derivations are ineligible for `#35` partial output projection).
- **No universal minimality claim**: separate executor vs metadata vs transfer counts verify key `#41` / `#35` / early-cutoff skip invariants, not globally optimal read/schedule counts across all possible DAGs.
- **No unrestricted Nix compatibility or `#19` incremental Nickel eval reuse**: `TaskGraph` and the Comment 1 Nickel fixtures cover Mantle's bounded derivation and composition semantics, not full Nixpkgs evaluation or incremental Nickel evaluator caching (`#19`).
