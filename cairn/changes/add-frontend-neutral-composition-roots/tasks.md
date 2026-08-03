# Tasks

## Phase 1: Architecture and baseline

- [x] [serial] I1 Add ADR 0057 for concrete frontend-neutral plans, transport-independent identity, castore-root identity, optional adapters, and deferred execution or deployment. r[composition_roots.frontend_neutral_plan] r[composition_roots.optional_adapters]
  - Evidence: `adr/0057-keep-composition-plans-concrete-and-frontend-neutral.md`.
- [ ] [serial] V1 Before core changes, run the focused existing castore node, directory, store handle, action-result core, build request, and execution-profile tests. Record exact commands and results in `evidence/baseline-tests.md`. r[composition_roots.pure_bounded_core] r[composition_roots.castore_realization]

## Phase 2: Pure model and identity

- [ ] [serial] I2 Add a pure composition core with versioned plan, derived binding-reference, mount-target, collision-decision, realization-policy, outcome, and receipt-preimage types. r[composition_roots.pure_bounded_core]
- [ ] [serial] I3 Add strict logical-path normalization and stable invalid-path diagnostics without host filesystem access. r[composition_roots.logical_path_safety]
- [ ] [serial] I4 Add domain-separated BLAKE3 identities for normalized merge semantics and separate realization policy. Derive binding references from roots and mounts. Exclude raw encoding, list order, store prefixes, source envelopes, caller labels, and physical paths from plan identity. r[composition_roots.canonical_plan_identity]
- [ ] [serial] I5 Add deterministic recursive merge planning with directory union, identical-leaf deduplication, and exact explicit collision decisions. r[composition_roots.explicit_conflicts]
- [ ] [parallel] I6 Add positive tests for root and nested mounts, reordered bindings and decisions, caller-label independence, separate policy identity, recursive directory union, identical-leaf deduplication, and explicit leaf replacement. r[composition_roots.canonical_plan_identity] r[composition_roots.explicit_conflicts]
- [ ] [parallel] I7 Add negative tests for duplicate bindings, unsafe paths, malformed refs, every named limit, missing snapshots, unresolved collisions, duplicate decisions, stale decisions, and non-contributing winners. r[composition_roots.logical_path_safety] r[composition_roots.explicit_conflicts]
- [ ] [parallel] I8 Add architecture tests that reject filesystem, store, environment, process, network, clock, random, serialization-shell, and presentation dependencies from the pure core. r[composition_roots.pure_bounded_core]

## Phase 3: Castore realization shell

- [ ] [serial] I9 Add a thin shell that loads complete bounded castore snapshots for every input root and rejects missing or inconsistent objects before persistence. r[composition_roots.castore_realization]
- [ ] [serial] I10 Translate planned directory facts into castore nodes, persist them in dependency order, and recheck the resulting object graph. r[composition_roots.castore_realization]
- [ ] [serial] I11 Emit a deterministic realization receipt that binds plan, realization policy, inputs, merge policy, outcomes, limit usage, and the resulting castore root. r[composition_roots.realization_receipt]
- [ ] [parallel] I12 Add positive store tests for complete multi-root realization, repeat realization, shared object reuse, and output-root equality after input reordering. r[composition_roots.castore_realization]
- [ ] [parallel] I13 Add negative store tests for missing roots, missing child directories, missing blobs, corrupt nodes, persistence failure, root-recheck failure, and no successful receipt after any failure. r[composition_roots.castore_realization] r[composition_roots.realization_receipt]

## Phase 4: Experimental generic surface

- [ ] [serial] I14 Add an experimental generic CLI or API shell that decodes one bounded projection into the core model and emits plan and realization results. r[composition_roots.optional_adapters] r[composition_roots.experimental_boundary]
- [ ] [parallel] I15 Add CLI fixtures that prove equivalent projection order has one `plan_ref`, store-prefix labels cannot enter identity, malformed projections fail, and no OnixOS or Preserves runtime is required. r[composition_roots.canonical_plan_identity] r[composition_roots.optional_adapters]
- [ ] [parallel] I16 Add boundary checks that reject OnixOS package, provider, inventory, activation, deployment, and rollback semantics from the core and generic CLI contract. r[composition_roots.frontend_neutral_plan]
- [ ] [serial] I17 Document the plan schema, result receipt, conflict model, metadata support matrix, compatibility behavior, adapter boundary, and non-claims. r[composition_roots.realization_receipt] r[composition_roots.experimental_boundary]

## Phase 5: Validation and lifecycle evidence

- [ ] [serial] V2 Rerun every V1 command after implementation. Run focused positive and negative core, store, and CLI suites, and record exact output in `evidence/focused-tests.md`. r[composition_roots.pure_bounded_core] r[composition_roots.castore_realization]
- [ ] [serial] V3 Run formatting, focused Clippy, Tiger Style checks, architecture checks, and `git diff --check`. Record exact output in `evidence/quality-checks.md`. r[composition_roots.pure_bounded_core]
- [ ] [serial] V4 Run Cairn validation, proposal, design, and tasks gates, plus Tracey coverage. Record exact output before sync and archive. r[composition_roots.realization_receipt]
- [ ] [serial] V5 Review whether stable implementation evidence justifies separate Cairn changes for path-free action results, composed-root execution, Kamacite adaptation, or OnixOS lowering. Do not create those integrations by implication. r[composition_roots.optional_adapters] r[composition_roots.experimental_boundary]
