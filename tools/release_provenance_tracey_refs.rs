// Accepted release-provenance Tracey bridge.
//
// This file links only requirements already synchronized into
// `cairn/specs/release-provenance/spec.md`. Active change requirements remain
// outside this accepted-spec profile until Cairn sync and archive complete.
// Each section names the implementation, tests, and archived validation that
// were inspected before adding these references.
//
// These links support traceability for bounded identity, linkage, policy, and
// filesystem-confinement claims. They do not prove artifact correctness,
// behavioral correctness, semantic equivalence, verifier soundness, deployment
// safety, reproducibility beyond recorded fixtures, or release eligibility.

// Source-observation release binding.
// Implementation and tests: `crates/crunch-release-core/src/manifest.rs`,
// `src/release_evidence.rs`, and `tests/release_cli.rs`.
// r[impl mantle.release_provenance.source_observation_binding]
// r[verify mantle.release_provenance.source_observation_signature_boundary]

// Valence-validated stack provenance.
// Implementation and tests: `crates/crunch-release-core/src/manifest.rs`,
// `src/release_evidence.rs`, `src/release_cmd.rs`, and `tests/release_cli.rs`.
// Archived review: `cairn/archive/1970-01-01-valence-validated-release-provenance/`.
// r[impl mantle.release_provenance.valence_required_policy]
// r[verify mantle.release_provenance.valence_required_policy.optional_absent]
// r[verify mantle.release_provenance.valence_required_policy.required_valid]
// r[verify mantle.release_provenance.valence_required_policy.required_missing]
// r[impl mantle.release_provenance.valence_receipt_binding]
// r[verify mantle.release_provenance.valence_receipt_binding.sidecar_digest]
// r[verify mantle.release_provenance.valence_receipt_binding.valence_receipt]
// r[verify mantle.release_provenance.valence_receipt_binding.binary_identity]
// r[verify mantle.release_provenance.valence_receipt_binding.stale]
// r[impl mantle.release_provenance.opaque_boundary]
// r[verify mantle.release_provenance.opaque_boundary.visible]
// r[verify mantle.release_provenance.opaque_boundary.overclaim]
// r[impl mantle.release_provenance.fixture_matrix]
// r[verify mantle.release_provenance.fixture_matrix.positive]
// r[verify mantle.release_provenance.fixture_matrix.negative]

// Required Onix stack profile.
// Implementation and tests: `crates/crunch-release-core/src/manifest.rs`,
// `src/release_cmd.rs`, `src/release_evidence.rs`, and operator documentation.
// Archived validation:
// `cairn/archive/2026-07-09-required-stack-provenance-profile/evidence/manual-validation.md`.
// r[impl mantle.release_provenance.stack_profile.required]
// r[verify mantle.release_provenance.stack_profile.positive]
// r[verify mantle.release_provenance.stack_profile.required_absent]
// r[impl mantle.release_provenance.stack_profile.constants]
// r[verify mantle.release_provenance.stack_profile.constants.match]
// r[verify mantle.release_provenance.stack_profile.constants.drift]
// r[impl mantle.release_provenance.stack_profile.negative]
// r[verify mantle.release_provenance.stack_profile.negative.invalid]
// r[impl mantle.release_provenance.stack_profile.docs]
// r[verify mantle.release_provenance.stack_profile.docs.generic]
// r[verify mantle.release_provenance.stack_profile.docs.boundary]
// r[impl mantle.release_provenance.stack_profile.validation]
// r[verify mantle.release_provenance.stack_profile.validation.fixtures]

// Preserves release-evidence carriers.
// Implementation and tests: `src/preserves_release_carrier.rs`.
// Archived validation:
// `cairn/archive/2026-07-09-preserves-release-evidence-carriers/evidence/manual-validation.md`.
// r[impl mantle.release_provenance.preserves_carriers.contract]
// r[verify mantle.release_provenance.preserves_carriers.fixtures.positive]
// r[verify mantle.release_provenance.preserves_carriers.fixtures.negative]
// r[impl mantle.release_provenance.preserves_carriers.validation]
// r[impl mantle.release_provenance.preserves_carriers.docs]
// r[verify mantle.release_provenance.preserves_carriers.final_validation]

// Cairn release-evidence handoff rows.
// Implementation and tests: `src/cairn_release_handoff.rs`,
// `src/release_evidence.rs`, `crates/crunch-release-core`, release CLI fixtures,
// and `.github/workflows/flake-check.yml`.
// Archived validation:
// `cairn/archive/2026-07-09-mantle-cairn-release-evidence-handoff/evidence/manual-validation.md`
// and `cairn/archive/2026-07-14-enforce-hermetic-release-handoff/evidence/validation.md`.
// r[impl mantle.release_provenance.cairn_evidence_handoff.contract]
// r[impl mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
// r[impl mantle.release_provenance.cairn_evidence_handoff.production_wiring]
// r[impl mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
// r[impl mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
// r[verify mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
// r[verify mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
// r[impl mantle.release_provenance.cairn_evidence_handoff.validation]
// r[impl mantle.release_provenance.cairn_evidence_handoff.docs]
// r[verify mantle.release_provenance.cairn_evidence_handoff.flake_check_ci]
// r[verify mantle.release_provenance.cairn_evidence_handoff.final_validation]

// Nix evidence normalization.
// Implementation and tests: `src/nix_evidence_core.rs`.
// Archived validation: `cairn/archive/2026-07-09-nix-evidence-core/evidence/manual-validation.md`.
// r[impl mantle.release_provenance.nix_evidence_core.contract]
// r[verify mantle.release_provenance.nix_evidence_core.fixtures.positive]
// r[verify mantle.release_provenance.nix_evidence_core.fixtures.negative]
// r[impl mantle.release_provenance.nix_evidence_core.validation]
// r[verify mantle.release_provenance.nix_evidence_core.adapters]
// r[impl mantle.release_provenance.nix_evidence_core.docs]
// r[verify mantle.release_provenance.nix_evidence_core.final_validation]

// Release capability boundaries.
// Implementation and tests: `src/release_capability.rs` and release shell users.
// Archived validation:
// `cairn/archive/2026-07-09-adopt-cap-std-release-boundaries/evidence/manual-validation.md`.
// r[impl mantle.release_provenance.cap_std_boundary.dependency]
// r[impl mantle.release_provenance.cap_std_boundary.root_wrappers]
// r[verify mantle.release_provenance.cap_std_boundary.tests.positive]
// r[impl mantle.release_provenance.cap_std_boundary.conversion]
// r[verify mantle.release_provenance.cap_std_boundary.tests.negative]
// r[impl mantle.release_provenance.cap_std_boundary.docs]
// r[verify mantle.release_provenance.cap_std_boundary.validation]

// Function-address evidence policy and bundle binding.
// Implementation and tests: `crates/crunch-release-core/src/manifest.rs`,
// `src/release_evidence.rs`, and `tests/release_cli.rs`.
// Archived review: `cairn/archive/1970-01-01-function-address-release-evidence/`.
// r[impl mantle.release_provenance.function_address_evidence.policy]
// r[verify mantle.release_provenance.function_address_evidence.policy.optional_absent]
// r[verify mantle.release_provenance.function_address_evidence.policy.required_valid]
// r[verify mantle.release_provenance.function_address_evidence.policy.required_missing]
// r[impl mantle.release_provenance.function_address_evidence.binding]
// r[verify mantle.release_provenance.function_address_evidence.binding.digests]
// r[verify mantle.release_provenance.function_address_evidence.binding.binary_source]
// r[verify mantle.release_provenance.function_address_evidence.binding.stale]
// r[impl mantle.release_provenance.function_address_evidence.validation]
// r[verify mantle.release_provenance.function_address_evidence.validation.opaque]
// r[verify mantle.release_provenance.function_address_evidence.validation.boundary]
// r[impl mantle.release_provenance.function_address_evidence.fixtures.positive]
// r[verify mantle.release_provenance.function_address_evidence.fixtures.positive.modes]
// r[impl mantle.release_provenance.function_address_evidence.fixtures.negative]
// r[verify mantle.release_provenance.function_address_evidence.fixtures.negative.required]
// r[impl mantle.release_provenance.function_address_evidence.docs]
// r[verify mantle.release_provenance.function_address_evidence.docs.boundary]
// r[impl mantle.release_provenance.function_address_evidence.final_validation]
// r[verify mantle.release_provenance.function_address_evidence.final_validation.fixtures]

// Release tree-copy confinement.
// Implementation and tests: `crates/crunch-release-core/src/tree_copy.rs`,
// `src/release_tree_copy.rs`, `src/release_evidence.rs`, and release CLI tests.
// Archived validation:
// `cairn/archive/2026-07-12-confine-release-bundle-tree-copy/evidence/validation.md`.
// r[impl mantle.release_provenance.bundle_tree_copy.no_follow]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.positive]
// r[impl mantle.release_provenance.bundle_tree_copy.plan]
// r[verify mantle.release_provenance.bundle_tree_copy.plan.invalid]
// r[impl mantle.release_provenance.bundle_tree_copy.destination_confinement]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
// r[impl mantle.release_provenance.bundle_tree_copy.symlink_policy]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
// r[impl mantle.release_provenance.bundle_tree_copy.validation]
// r[verify mantle.release_provenance.bundle_tree_copy.validation.production]

// Generic opaque evidence sidecars.
// Implementation and tests: `crates/crunch-release-core/src/opaque_evidence.rs`
// and `crates/crunch-release-core/src/manifest.rs`.
// Archived validation:
// `cairn/archive/2026-07-12-opaque-evidence-sidecar-binding/evidence/validation.md`.
// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.contract]
// r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.links]
// r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core]
// r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
// r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.negative]

// Trellis proof release sidecars.
// Implementation and tests: `crates/crunch-release-core/src/opaque_evidence.rs`,
// `crates/crunch-release-core/src/trellis_proof_binding.rs`,
// `crates/crunch-release-core/src/manifest.rs`, and the positive/negative fixtures.
// Validation: `cairn/archive/2026-07-14-trellis-proof-release-sidecars/evidence/implementation.md`
// and Valence archived authority commit `27b8b212`.
// r[impl mantle.release_provenance.trellis_proof_sidecars.profile]
// r[verify mantle.release_provenance.trellis_proof_sidecars.positive]
// r[impl mantle.release_provenance.trellis_proof_sidecars.links]
// r[verify mantle.release_provenance.trellis_proof_sidecars.negative]
// r[impl mantle.release_provenance.trellis_proof_sidecars.opaque]
// r[verify mantle.release_provenance.trellis_proof_sidecars.validation]

// Final release-verification decision.
// Implementation and tests: `crates/crunch-release-core/src/verification_decision.rs`,
// `src/release_cmd.rs`, and `tests/release_cli.rs`.
// Archived validation:
// `cairn/archive/2026-07-12-defer-release-verification-success/evidence/validation.md`.
// r[impl mantle.release_provenance.verification_decision.complete]
// r[verify mantle.release_provenance.verification_decision.fixtures.positive]
// r[verify mantle.release_provenance.verification_decision.fixtures.negative]
// r[impl mantle.release_provenance.verification_decision.boundary]
// r[verify mantle.release_provenance.verification_decision.boundary.test]
// r[impl mantle.release_provenance.verification_decision.completeness]
// r[verify mantle.release_provenance.verification_decision.completeness.test]

// Function-address binding receipt schema.
// Implementation and tests: `crates/crunch-release-core/src/function_address_binding.rs`
// and the generated machine-contract fixtures.
// Archived validation:
// `cairn/archive/2026-07-12-function-address-release-binding-schema/evidence/validation.md`.
// r[impl mantle.release_provenance.function_address_binding_schema.contract]
// r[verify mantle.release_provenance.function_address_binding_schema.positive]
// r[impl mantle.release_provenance.function_address_binding_schema.cairn_mapping]
// r[verify mantle.release_provenance.function_address_binding_schema.validation]
// r[verify mantle.release_provenance.function_address_binding_schema.negative]
// r[verify mantle.release_provenance.function_address_binding_schema.render]

// Function-address binding CLI.
// Implementation and tests: `src/function_address_binding_cmd.rs`,
// `crates/crunch-release-core/src/function_address_binding.rs`, and release CLI fixtures.
// Archived validation:
// `cairn/archive/2026-07-12-function-address-release-binding-cli/evidence/validation.md`.
// r[impl mantle.release_provenance.function_address_binding_cli.command]
// r[verify mantle.release_provenance.function_address_binding_cli.positive]
// r[verify mantle.release_provenance.function_address_binding_cli.receipt.identity_domains]
// r[impl mantle.release_provenance.function_address_binding_cli.shell]
// r[verify mantle.release_provenance.function_address_binding_cli.receipt]
// r[verify mantle.release_provenance.function_address_binding_cli.shell.replacement]
// r[verify mantle.release_provenance.function_address_binding_cli.negative]
// r[verify mantle.release_provenance.function_address_binding_cli.validation]

// Function-address Preserves sidecars.
// Implementation and tests: the opaque-evidence, manifest, binding-core, and CLI
// files named in the archived validation.
// Archived validation:
// `cairn/archive/2026-07-12-function-address-preserves-sidecars/evidence/validation.md`.
// r[impl mantle.release_provenance.function_address_preserves_sidecars.contract]
// r[verify mantle.release_provenance.function_address_preserves_sidecars.positive]
// r[impl mantle.release_provenance.function_address_preserves_sidecars.opaque]
// r[verify mantle.release_provenance.function_address_preserves_sidecars.validation]
// r[impl mantle.release_provenance.function_address_preserves_sidecars.json_projection]
// r[verify mantle.release_provenance.function_address_preserves_sidecars.negative]

// Atomic release publication.
// Implementation and tests: `crates/crunch-release-core/src/publication.rs`,
// `src/release_publication.rs`, `src/release_evidence.rs`, and release CLI tests.
// Archived validation:
// `cairn/archive/2026-07-12-publish-release-bundles-atomically/evidence/validation.md`.
// r[impl mantle.release_provenance.bundle_publication.atomic_commit]
// r[verify mantle.release_provenance.bundle_publication.fixtures.positive]
// r[verify mantle.release_provenance.bundle_publication.fixtures.negative.race]
// r[impl mantle.release_provenance.bundle_publication.staging_validation]
// r[verify mantle.release_provenance.bundle_publication.fixtures.negative.verification]
// r[impl mantle.release_provenance.bundle_publication.failure_isolation]
// r[verify mantle.release_provenance.bundle_publication.retry]
// r[verify mantle.release_provenance.bundle_publication.stale_stage]
// r[impl mantle.release_provenance.bundle_publication.boundary]
// r[verify mantle.release_provenance.bundle_publication.boundary.test]
// r[impl mantle.release_provenance.bundle_publication.validation]
// r[verify mantle.release_provenance.bundle_publication.validation.visibility]

// Genuine deterministic rebuild admission.
// Implementation and tests: `crates/crunch-release-core/src/genuine_rebuild.rs`,
// `src/release_reproducibility.rs`, and release verification/witness fixtures.
// Archived validation:
// `cairn/archive/2026-07-12-require-genuine-release-rebuild-proofs/evidence/README.md`.
// r[impl mantle.release_provenance.deterministic_rebuild_admission.contract]
// r[verify mantle.release_provenance.deterministic_rebuild_admission.legacy]
// r[impl mantle.release_provenance.deterministic_rebuild_admission.validation]
// r[verify mantle.release_provenance.deterministic_rebuild_admission.fixtures.negative]

// Content-bound requirement evidence.
// Implementation and tests: `crates/crunch-release-core/src/content_bound_requirements.rs`,
// `src/content_bound_requirement_evidence.rs`, `src/release_evidence.rs`, and
// `src/release_cmd.rs`. The dedicated bridge remains in
// `tools/content_bound_release_requirement_tracey_refs.rs`.
// Archived validation:
// `cairn/archive/2026-07-27-bind-release-requirement-evidence/`.
// r[impl mantle.release_provenance.content_bound_requirement_coverage]
// r[verify mantle.release_provenance.content_bound_evidence_manifest]
// r[verify mantle.release_provenance.legacy_coverage_boundary]

// Chaptered release transport.
// Implementation and tests: `crates/crunch-release-core/src/chapter_transport.rs`,
// `src/release_chapter_transport.rs`, `src/release_tree_copy.rs`,
// `src/release_capability.rs`, `src/release_cmd.rs`, and `src/main.rs`.
// Validation: `cairn/archive/2026-08-03-adopt-chapter-tgz-release-transport/`.
// r[impl mantle.release_provenance.chapter_transport.optional]
// r[verify mantle.release_provenance.chapter_transport.optional]
// r[impl mantle.release_provenance.chapter_transport.plan]
// r[verify mantle.release_provenance.chapter_transport.plan]
// r[impl mantle.release_provenance.chapter_transport.pack]
// r[verify mantle.release_provenance.chapter_transport.pack]
// r[impl mantle.release_provenance.chapter_transport.receipt]
// r[verify mantle.release_provenance.chapter_transport.receipt]
// r[impl mantle.release_provenance.chapter_transport.inspect]
// r[verify mantle.release_provenance.chapter_transport.inspect]
// r[impl mantle.release_provenance.chapter_transport.unpack]
// r[verify mantle.release_provenance.chapter_transport.unpack]
// r[impl mantle.release_provenance.chapter_transport.validation]
// r[verify mantle.release_provenance.chapter_transport.validation]
