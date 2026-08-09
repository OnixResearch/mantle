# Tasks

## Phase 1: Architecture and baseline

- [x] [serial] I1 Add ADR 0057 for concrete frontend-neutral plans, transport-independent identity, castore-root identity, optional adapters, and deferred execution or deployment. r[composition_roots.frontend_neutral_plan] r[composition_roots.optional_adapters]
  - Evidence: `adr/0057-keep-composition-plans-concrete-and-frontend-neutral.md`.
- [x] [serial] V1 Before core changes, run the focused existing castore node, directory, store handle, action-result core, build request, and execution-profile tests. Record exact commands and results in `evidence/baseline-tests.md`. r[composition_roots.pure_bounded_core] r[composition_roots.castore_realization]
  - Evidence: all six baseline commands passed; `evidence/baseline-tests.md` records the exact commands, output, and status.

## Phase 2: Pure model and identity

- [x] [serial] I2 Add a pure composition core with versioned plan, derived binding-reference, mount-target, collision-decision, realization-policy, outcome, and receipt-preimage types. r[composition_roots.pure_bounded_core]
- [x] [serial] I3 Add strict logical-path normalization and stable invalid-path diagnostics without host filesystem access. r[composition_roots.logical_path_safety]
- [x] [serial] I4 Add domain-separated BLAKE3 identities for normalized merge semantics and separate realization policy. Derive binding references from roots and mounts. Exclude raw encoding, list order, store prefixes, source envelopes, caller labels, and physical paths from plan identity. r[composition_roots.canonical_plan_identity]
- [x] [serial] I5 Add deterministic recursive merge planning with directory union, identical-leaf deduplication, and exact explicit collision decisions. r[composition_roots.explicit_conflicts]
- [x] [parallel] I6 Add positive tests for root and nested mounts, reordered bindings and decisions, caller-label independence, separate policy identity, recursive directory union, identical-leaf deduplication, and explicit leaf replacement. r[composition_roots.canonical_plan_identity] r[composition_roots.explicit_conflicts]
- [x] [parallel] I7 Add negative tests for duplicate bindings, unsafe paths, malformed refs, every named limit, missing snapshots, unresolved collisions, duplicate decisions, stale decisions, and non-contributing winners. r[composition_roots.logical_path_safety] r[composition_roots.explicit_conflicts]
- [x] [parallel] I8 Add architecture tests that reject filesystem, store, environment, process, network, clock, random, serialization-shell, and presentation dependencies from the pure core. r[composition_roots.pure_bounded_core]

## Phase 3: Castore realization shell

- [x] [serial] I9 Add a thin shell that loads complete bounded castore snapshots for every input root and rejects missing or inconsistent objects before persistence. r[composition_roots.castore_realization]
- [x] [serial] I10 Translate planned directory facts into castore nodes, persist them in dependency order, and recheck the resulting object graph. r[composition_roots.castore_realization]
- [x] [serial] I11 Emit a deterministic realization receipt that binds plan, realization policy, inputs, merge policy, outcomes, limit usage, and the resulting castore root. r[composition_roots.realization_receipt]
- [x] [parallel] I12 Add positive store tests for complete multi-root realization, repeat realization, shared object reuse, and output-root equality after input reordering. r[composition_roots.castore_realization]
- [x] [parallel] I13 Add negative store tests for missing roots, missing child directories, missing blobs, corrupt nodes, persistence failure, root-recheck failure, and no successful receipt after any failure. r[composition_roots.castore_realization] r[composition_roots.realization_receipt]

## Phase 4: Experimental generic surface

- [x] [serial] I14 Add an experimental generic CLI or API shell that decodes one bounded projection into the core model and emits plan and realization results. r[composition_roots.optional_adapters] r[composition_roots.experimental_boundary]
- [x] [parallel] I15 Add CLI fixtures that prove equivalent projection order has one `plan_ref`, store-prefix labels cannot enter identity, malformed projections fail, and no OnixOS or Preserves runtime is required. r[composition_roots.canonical_plan_identity] r[composition_roots.optional_adapters]
- [x] [parallel] I16 Add boundary checks that reject OnixOS package, provider, inventory, activation, deployment, and rollback semantics from the core and generic CLI contract. r[composition_roots.frontend_neutral_plan]
- [x] [serial] I17 Document the plan schema, result receipt, conflict model, metadata support matrix, compatibility behavior, adapter boundary, and non-claims. r[composition_roots.realization_receipt] r[composition_roots.experimental_boundary]

## Phase 5: Validation and lifecycle evidence

- [x] [serial] V2 Rerun every V1 command after implementation. Run focused positive and negative core, store, and CLI suites, and record exact output in `evidence/focused-tests.md`. r[composition_roots.pure_bounded_core] r[composition_roots.castore_realization]
  - Commands: `nix develop -c cargo test -q -p snix-castore node --lib`; `nix develop -c cargo test -q -p snix-castore directory --lib`; `nix develop -c cargo test -q -p crunch-store handle::tests:: --lib`; `nix develop -c cargo test -q -p crunch-action-result-core`; `nix develop -c cargo test -q -p crunch-build build_request::tests:: --lib`; `nix develop -c cargo test -q -p crunch-build execution_profile --lib`; `nix develop -c cargo test -q -p crunch-composition-core`; `nix develop -c cargo check -q -p crunch-composition-core --target wasm32-unknown-unknown`; `nix develop -c cargo test -q -p crunch-store composition::tests:: --lib`; `nix develop -c cargo test -q -p mantle --test composition_root_cli`.
  - Evidence: the six baseline commands passed again; composition core ran 10 unit and 2 boundary tests, store ran 4 focused tests, CLI ran 3 tests, and the no-std wasm check passed.
- [x] [serial] V3 Run formatting, focused Clippy, Tiger Style checks, architecture checks, and `git diff --check`. Record exact output in `evidence/quality-checks.md`. r[composition_roots.pure_bounded_core]
  - Commands: `nix develop -c cargo fmt --check -p crunch-composition-core -p crunch-store`; `CARGO_TARGET_DIR=/tmp/mantle-composition-clippy-quality-20260809 nix develop -c cargo clippy -q -p crunch-composition-core --all-targets --no-deps -- -D warnings`; `CARGO_TARGET_DIR=/tmp/mantle-composition-clippy-quality-20260809 nix develop -c cargo clippy -q -p crunch-store --lib --no-deps -- -D warnings`; `CARGO_TARGET_DIR=/tmp/mantle-composition-clippy-quality-20260809 nix develop -c cargo clippy -q -p mantle --bin mantle --test composition_root_cli --no-deps -- -D warnings`; `nix develop -c nickel typecheck config/composition-roots/authority.ncl`; `nix develop -c ./scripts/check-operator-command-contract.sh`; `git diff --check`; `nix develop -c cargo fmt --check -p mantle -v`; `CARGO_TARGET_DIR=/tmp/mantle-composition-tiger-target nix run .#tigerstyle -- check -- -p crunch-composition-core --all-targets`.
  - Evidence: focused formatting, product-owned Clippy, Nickel, operator-contract, architecture, and diff checks passed. Broad formatting and whole-tree Tiger Style preserved the existing fixed-point and GC/overlay findings.
- [x] [serial] V4 Run Cairn validation, proposal, design, and tasks gates, plus Tracey coverage. Record exact output before sync and archive. r[composition_roots.realization_receipt]
  - Commands: legacy Cairn `validate --root .`; `gate proposal|design|tasks add-frontend-neutral-composition-roots --root .`; `tracey coverage --root .`.
  - Evidence: `evidence/cairn-pre-archive.md` records valid repository output, three `PASS` gates, and Tracey `155/155 referenced`.
- [x] [serial] V5 Review whether stable implementation evidence justifies separate Cairn changes for path-free action results, composed-root execution, Kamacite adaptation, or OnixOS lowering. Do not create those integrations by implication. r[composition_roots.optional_adapters] r[composition_roots.experimental_boundary]
  - Evidence: `evidence/follow-up-review.md` keeps all four integration surfaces deferred to separate accepted changes.
