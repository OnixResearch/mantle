## Implementation

- [x] [serial] **I1** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core] Replace direct parse-and-register flow with private parsed, validated, identity-resolved, and registry-ready states.
  Evidence: `dynamic.rs` exposes only staged constructors to the Worker, and the registry accepts only the final private state.
- [x] [serial] **I2** r[dynamic_derivation_admission.complete_parent_identity] [covers=dynamic_derivation_admission.complete_parent_identity] Require explicit Mantle BLAKE3 facts for every direct parent and remove all-zero fallback identity.
  Evidence: missing, duplicate, conflicting, unexpected, wrong-prefix, and wrong-domain facts return stable errors before identity resolution.
- [x] [serial] **I3** r[dynamic_derivation_admission.versioned_forms] [covers=dynamic_derivation_admission.versioned_forms] Add bounded iterative support for traditional `Derive` and versioned `DrvWithVersion("xp-dyn-drv",...)` forms.
  Evidence: named limits cover bytes, fields, collections, parents, nodes, depth, and parser collections without recursive traversal.
- [x] [serial] **I4** r[dynamic_derivation_admission.registry_boundary] [covers=dynamic_derivation_admission.registry_boundary] Split Worker observations and effects from pure admission decisions.
  Evidence: castore reads and registry observations precede batch preflight; registry, goal, waiter, queue, and report effects follow it.
- [x] [serial] **I5** r[dynamic_derivation_admission.registry_boundary] [covers=dynamic_derivation_admission.registry_boundary] Make exact duplicates idempotent and reject path collisions before batch mutation.
  Evidence: batch selection keeps one exact representative and rejects different full identities for one path.
- [x] [serial] **I6** r[dynamic_derivation_admission.compatibility] [covers=dynamic_derivation_admission.compatibility] Preserve covered traditional HDM and configured-prefix path behavior.
  Evidence: compatibility tests compare the new traditional result with the prior native algorithm and cover a custom logical prefix.
- [x] [serial] **I7** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core] Add `scripts/check-dynamic-admission-boundary.rs` with positive and negative self-tests.
  Evidence: the guard rejects effects, registry types, `nix-derivation`, panic helpers, sentinel digests, and sentinel integer fallbacks in the core.
- [x] [serial] **I8** r[dynamic_derivation_admission.claim_boundary] [covers=dynamic_derivation_admission.claim_boundary] Update ADR 0002, README guidance, operator diagnostics, rollback facts, and Tracey references.
  Evidence: `docs/dynamic-derivation-admission.md` lists forms, limits, blockers, hash domains, rollback, validation, and non-claims.

## Validation

- [x] [serial] **V1** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core] [evidence=evidence/implementation-validation.md] Run `nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-dynamic-admission-target cargo test -p crunch-build --lib --tests`.
  Evidence summary: 674 unit tests and one API integration test passed.
- [x] [serial] **V2** r[dynamic_derivation_admission.complete_parent_identity] [covers=dynamic_derivation_admission.complete_parent_identity] [evidence=evidence/implementation-validation.md] Run focused missing, duplicate, conflict, wrong-prefix, wrong-domain, and unexpected-parent tests.
  Evidence summary: every incomplete or invalid parent-fact class failed before registry-ready state.
- [x] [serial] **V3** r[dynamic_derivation_admission.versioned_forms] [covers=dynamic_derivation_admission.versioned_forms] [evidence=evidence/implementation-validation.md] Run positive and negative versioned parser tests.
  Evidence summary: supported nested and custom-prefix forms passed; unknown, malformed, empty, unsupported-output, over-size, and over-depth forms failed.
- [x] [serial] **V4** r[dynamic_derivation_admission.compatibility] [covers=dynamic_derivation_admission.compatibility] [evidence=evidence/implementation-validation.md] Run traditional identity and configurable-prefix compatibility tests.
  Evidence summary: covered HDM and path results match the prior algorithm without mixed-prefix acceptance.
- [x] [serial] **V5** r[dynamic_derivation_admission.registry_boundary] [covers=dynamic_derivation_admission.registry_boundary] [evidence=evidence/implementation-validation.md] Run Worker duplicate, collision, missing-parent, streaming, and build tests.
  Evidence summary: exact duplicates insert once; rejected missing-parent and collision paths leave Worker and registry state unchanged.
- [x] [serial] **V6** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core,dynamic_derivation_admission.registry_boundary] [evidence=evidence/implementation-validation.md] Run `cargo -Zscript scripts/check-dynamic-admission-boundary.rs --self-test`, the source guard, and `scripts/check-nix-derivation-boundary.rs`.
  Evidence summary: positive and negative guard fixtures passed, and no compatibility dependency leaked into native identity code.
- [x] [serial] **V7** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core] [evidence=evidence/implementation-validation.md] Run focused strict Clippy, the first-party Clippy gate, focused Tiger Style, and workspace tests.
  Evidence summary: changed-package Clippy passed; changed dynamic source has no Tiger finding; broad unrelated findings are recorded exactly.
- [x] [serial] **V8** r[dynamic_derivation_admission.claim_boundary] [covers=dynamic_derivation_admission.claim_boundary] [evidence=evidence/implementation-validation.md] Run targeted rustfmt, `cargo fmt --all -- --check`, and `git diff --check`.
  Evidence summary: changed Rust files and diff checks passed; only the known unrelated fixed-point shell drift remains repo-wide.
- [x] [serial] **V9** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core,dynamic_derivation_admission.complete_parent_identity,dynamic_derivation_admission.versioned_forms,dynamic_derivation_admission.registry_boundary,dynamic_derivation_admission.compatibility,dynamic_derivation_admission.claim_boundary] [evidence=evidence/cairn-validation.log] Run strict Cairn validation and proposal, design, and tasks gates.
  Evidence summary: strict validation, proposal gate, design gate, tasks gate, and completeness review passed without findings.
- [x] [serial] **V10** r[dynamic_derivation_admission.claim_boundary] [covers=dynamic_derivation_admission.claim_boundary] [evidence=evidence/implementation-validation.md] Run `nix build .#crunch -L` and validate the typed Nickel receipt.
  Evidence summary: release compilation completed; the known seccomp listener test blocked Nix completion after 173 passes; Nickel receipt export passed.

## Hardening and review

- [x] [serial] **H1** r[dynamic_derivation_admission.staged_core] [covers=dynamic_derivation_admission.staged_core,dynamic_derivation_admission.complete_parent_identity,dynamic_derivation_admission.versioned_forms,dynamic_derivation_admission.registry_boundary,dynamic_derivation_admission.compatibility,dynamic_derivation_admission.claim_boundary] [evidence=evidence/implementation-validation.md] Review incomplete facts, deep trees, malformed versions, unsupported outputs, mixed prefixes, hash-domain crossings, duplicates, collisions, and rollback.
  Evidence summary: the portfolio review found and fixed a custom-prefix projection defect, a duplicate-batch partial-mutation defect, and a weak output fixture.
