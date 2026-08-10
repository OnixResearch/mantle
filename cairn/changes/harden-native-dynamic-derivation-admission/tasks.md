# Tasks

## 1. Freeze current behavior

- [ ] [serial] 1.1 Run existing `crunch-build` dynamic derivation and Worker tests before core changes. r[dynamic_derivation_admission.compatibility]
- [ ] [serial] 1.2 Record traditional-form identities, configured-prefix paths, parent lookup behavior, registry mutations, and scheduler outcomes. r[dynamic_derivation_admission.compatibility]
- [ ] [serial] 1.3 Add an active negative fixture that demonstrates the current unknown-parent zero-hash fallback. r[dynamic_derivation_admission.complete_parent_identity]

## 2. Build the staged pure core

- [ ] [serial] 2.1 Add private parsed, validated, identity-resolved, and registry-ready dynamic derivation types. r[dynamic_derivation_admission.staged_core]
- [ ] [serial] 2.2 Move candidate, syntax, semantic, output-form, depth, parent, identity, and registration-plan decisions into pure functions. r[dynamic_derivation_admission.staged_core]
- [ ] [serial] 2.3 Keep castore reads, registry observations, tracing, mutation, goal creation, and dispatch in the Worker shell. r[dynamic_derivation_admission.staged_core]
- [ ] [serial] 2.4 Add stable typed blockers for every rejected transition. r[dynamic_derivation_admission.staged_core]

## 3. Require complete identity facts

- [ ] [serial] 3.1 Replace callback-based unknown-parent fallback with an explicit bounded parent-hash fact map. r[dynamic_derivation_admission.complete_parent_identity]
- [ ] [serial] 3.2 Reject missing, duplicate, conflicting, and wrong-prefix parent facts before identity calculation. r[dynamic_derivation_admission.complete_parent_identity]
- [ ] [serial] 3.3 Prove that every failed parent-resolution case leaves registry and scheduler state unchanged. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 3.4 Preserve Mantle-native BLAKE3 and configured-prefix identity for complete accepted graphs. r[dynamic_derivation_admission.compatibility]

## 4. Add bounded versioned forms

- [ ] [serial] 4.1 Detect traditional and declared versioned ATerm prefixes under one named byte bound. r[dynamic_derivation_admission.versioned_forms]
- [ ] [serial] 4.2 Model bounded recursive dynamic-input trees under one named depth policy. r[dynamic_derivation_admission.versioned_forms]
- [ ] [serial] 4.3 Reject unknown versions, excessive depth, empty requests, and unsupported output semantics before identity resolution. r[dynamic_derivation_admission.versioned_forms]
- [ ] [serial] 4.4 Add positive traversal tests and negative boundary tests without recursive stack dependence. r[dynamic_derivation_admission.versioned_forms]

## 5. Integrate registry and scheduler admission

- [ ] [serial] 5.1 Change registry insertion to accept only registry-ready dynamic derivations. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 5.2 Make duplicate discovery idempotent only for equal full admitted identity. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 5.3 Reject path collisions with different admitted identities before waiter or goal mutation. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 5.4 Update ADR 0002 and operator diagnostics with new blockers and non-claims. r[dynamic_derivation_admission.claim_boundary]

## 6. Validate positive and negative behavior

- [ ] [parallel] 6.1 Test valid traditional, versioned, known-parent, duplicate, and custom-prefix cases. r[dynamic_derivation_admission.compatibility]
- [ ] [parallel] 6.2 Test malformed, truncated, oversized, over-depth, unknown-version, unsupported-output, mixed-prefix, missing-parent, conflicting-parent, and collision cases. r[dynamic_derivation_admission.complete_parent_identity]
- [ ] [serial] 6.3 Prove through state snapshots that every negative case leaves registry, waiter, goal, and success-report state unchanged. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 6.4 Run `cargo fmt --all -- --check`. r[dynamic_derivation_admission.staged_core]
- [ ] [serial] 6.5 Run focused `crunch-build` dynamic and Worker tests. r[dynamic_derivation_admission.compatibility]
- [ ] [serial] 6.6 Run `cargo test --workspace`. r[dynamic_derivation_admission.registry_boundary]
- [ ] [serial] 6.7 Run `cargo clippy --workspace --all-targets -- -D warnings`. r[dynamic_derivation_admission.staged_core]
- [ ] [serial] 6.8 Run Cairn validation, requirement coverage, proposal gate, design gate, and tasks gate. r[dynamic_derivation_admission.claim_boundary]
