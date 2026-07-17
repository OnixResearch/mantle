// Mantle Tracey coverage bridge.
//
// Hermetic Cairn release handoff bridge.
//
// r[impl mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
// r[impl mantle.release_provenance.cairn_evidence_handoff.production_wiring]
// r[impl mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
// r[impl mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
// r[impl mantle.release_provenance.cairn_evidence_handoff.docs]
// r[impl mantle.release_provenance.cairn_evidence_handoff.flake_check_ci]
// r[impl mantle.build_correctness.onix_release_strict_hermeticity]
// r[impl mantle.build_correctness.hermetic_handoff.docs]
// r[impl mantle.build_correctness.source_root_capability]
// r[impl mantle.build_correctness.source_root_capability.boundary]
// The pure handoff/bundle-binding core lives in
// `crates/crunch-release-core/src/cairn_handoff.rs`; release assembly,
// remeasurement, CLI policy, and source-root host probes live under root `src/`.
//
// r[verify mantle.release_provenance.cairn_evidence_handoff.production_wiring]
// r[verify mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
// r[verify mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
// r[verify mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
// r[verify mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
// r[verify mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
// r[verify mantle.release_provenance.cairn_evidence_handoff.final_validation]
// r[verify mantle.release_provenance.cairn_evidence_handoff.flake_check_ci]
// r[verify mantle.build_correctness.onix_release_strict_hermeticity]
// r[verify mantle.build_correctness.hermetic_handoff.fixtures.positive]
// r[verify mantle.build_correctness.hermetic_handoff.fixtures.negative]
// r[verify mantle.build_correctness.hermetic_handoff.docs]
// r[verify mantle.build_correctness.source_root_capability]
// r[verify mantle.build_correctness.source_root_capability.boundary]
// Positive/negative core and root-package tests cover measured bytes,
// cross-bundle reuse, tampering, missing required handoffs, strict Onix
// requirements, the measured archived Cairn authentication prerequisite, and
// honest unsupported source-root self-build reporting. Mantle still does not
// claim independent producer-signature verification.
//
// Cairn's built-in tracey coverage rail currently scans `crates/` and `tools/`.
// Mantle's CLI/root-package implementation lives under top-level `src/`, so
// synced requirements implemented there need a small bridge until the coverage
// rail scans the package root directly.

// Atomic release publication bridge.
//
// r[impl mantle.release_provenance.bundle_publication.atomic_commit]
// r[impl mantle.release_provenance.bundle_publication.staging_validation]
// r[impl mantle.release_provenance.bundle_publication.failure_isolation]
// r[impl mantle.release_provenance.bundle_publication.stale_stage]
// r[impl mantle.release_provenance.bundle_publication.retry]
// The deterministic publication plan/state machine lives in
// `crates/crunch-release-core/src/publication.rs`. Capability-scoped sibling
// staging, manifest-last assembly, production verification, exact ownership
// marker quarantine, and Linux atomic no-replace commit live in
// `src/{release_evidence,release_publication,release_capability,release_tree_copy}.rs`.
//
// r[verify mantle.release_provenance.bundle_publication.fixtures.positive]
// r[verify mantle.release_provenance.bundle_publication.fixtures.negative.verification]
// r[verify mantle.release_provenance.bundle_publication.fixtures.negative.race]
// r[verify mantle.release_provenance.bundle_publication.validation.visibility]
// Verified by named core and production-path tests in
// `crates/crunch-release-core/src/publication.rs` and `src/release_evidence.rs`.
// Those tests cover deterministic identity, legal/illegal transitions,
// manifest-last visibility, source drift, verifier rejection, every named
// failpoint, current cleanup, exact stale-stage quarantine, unrecognized
// sibling preservation, fresh retry, and file/directory/symlink commit races.
// The evidence proves local visibility and no-clobber behavior, not filesystem
// crash durability, power-loss persistence, artifact correctness, or release
// eligibility.

// Function-address binding CLI bridge.
//
// r[impl mantle.release_provenance.function_address_binding_cli.command]
// r[impl mantle.release_provenance.function_address_binding_cli.shell]
// r[impl mantle.release_provenance.function_address_binding_cli.shell.replacement]
// r[impl mantle.release_provenance.function_address_binding_cli.receipt]
// r[impl mantle.release_provenance.function_address_binding_cli.receipt.identity_domains]
// The operator surface and bounded capability shell live in
// `src/{main,release_cmd,function_address_binding_cmd}.rs`; typed selection,
// identity-domain separation, validation, and deterministic receipt rendering
// live in `crates/crunch-release-core/src/function_address_binding.rs`.
//
// r[verify mantle.release_provenance.function_address_binding_cli.positive]
// r[verify mantle.release_provenance.function_address_binding_cli.negative]
// r[verify mantle.release_provenance.function_address_binding_cli.shell.replacement]
// r[verify mantle.release_provenance.function_address_binding_cli.receipt.identity_domains]
// r[verify mantle.release_provenance.function_address_binding_cli.validation]
// Verified by the positive optional/required and adversarial CLI fixtures in
// `tests/release_cli.rs`, machine-contract parity, and direct Cairn consumption
// of the CLI-generated receipt. This proves bounded bundle-local identity and
// linkage handling only, not upstream evidence semantics or release eligibility.

// Kernel-bundle OCI projection bridge.
//
// r[impl kernel_bundle_oci.projection]
// r[impl kernel_bundle_oci.admission]
// r[impl kernel_bundle_oci.layering]
// r[impl kernel_bundle_oci.digest_roles]
// r[impl kernel_bundle_oci.export]
// r[impl kernel_bundle_oci.import]
// Implemented by the pure deterministic core in `src/oci_projection.rs` and
// `src/oci_projection/`, plus the bounded filesystem/CAS shell in
// `src/oci_projection_shell.rs` and CLI dispatch in `src/artifact_cmd.rs`.
//
// r[verify kernel_bundle_oci.projection]
// r[verify kernel_bundle_oci.admission]
// r[verify kernel_bundle_oci.layering]
// r[verify kernel_bundle_oci.digest_roles]
// r[verify kernel_bundle_oci.export]
// r[verify kernel_bundle_oci.import]
// r[related kernel_bundle_oci.verification]
// Verified by positive and negative core/shell tests under the modules above,
// including deterministic rebuild, exact source-admission mismatch before CAS
// reads, exact/canonical layers, digest tampering, escaping links, reviewed
// Onix snapshots, atomic export, descriptor-first
// import, admitted reconstruction, and compatibility-only external import.
//
// r[related kernel_bundle_oci.reports]
// Local export/import reports remain distinct from the registry handoff reports;
// registry pull links back to the ordinary admitted import receipt rather than
// duplicating projection or CAS authority in the transport shell.
//
// Registry-backed OCI publication bridge.
//
// r[impl kernel_bundle_oci.registry_transport]
// r[impl kernel_bundle_oci.registry_admission]
// r[impl kernel_bundle_oci.registry_receipts]
// r[impl kernel_bundle_oci.registry_verification]
// Implemented by the pure target/manifest/linkage/accounting/receipt core in
// `src/oci_registry.rs`, the bounded ureq/filesystem shell in
// `src/oci_registry_shell.rs`, and public dispatch in `src/artifact_cmd.rs`.
// Existing `oci_projection` and `oci_projection_shell` remain authoritative for
// exact layout validation and admitted import.
//
// r[verify kernel_bundle_oci.registry_transport]
// r[verify kernel_bundle_oci.registry_admission]
// r[verify kernel_bundle_oci.registry_receipts]
// r[verify kernel_bundle_oci.registry_verification]
// Verified by positive/negative core and shell tests, the authenticated
// in-process registry cases in `tests/kernel_bundle_oci_registry_cli.rs`, the
// generated machine-contract rail, and catalog/docs drift checks in
// `tests/examples_inventory.rs` and `tests/examples_workflow_gallery.rs`.
// These checks cover dual immutable digest resolution, exact fresh-state
// admission, tag/blob/metadata drift, denied credentials, interrupted
// publication, content-addressed retry, redaction, bounds, and explicit
// non-claims without promoting registry possession into trust.

// Self-build source-closure bridge.
//
// r[impl bootstrap_inventory.self_build_source_closure]
// Implemented by the locked Cargo directory-source validator and fixed staging
// boundary in `src/self_build.rs`, stable diagnostic bound in
// `crates/crunch-build/src/distributed/remote_failure_debug.rs`, shared Linux
// no-replace shell in `src/linux_rename.rs`, and source-or-embedded Nickel
// import resolution in `src/remote_farm_config.rs`.
//
// r[verify bootstrap_inventory.self_build_source_closure]
// Verified by the source-staging/vendor-checksum unit tests, positive/negative
// no-clobber and typed Nickel config tests, the complete first-party quality and
// dependency rails, and the current fixed-point proof summary in
// `cairn/changes/restore-offline-self-build-source-closure/evidence/`.
// Evidence is limited to the explicit checkout-local vendor input and selected
// materialized-input proof transport; it does not imply fresh-clone offline
// completeness, compiler correctness, seed trust removal, or release eligibility.

// r[impl rust_package_planning.source_built_toolchain_closure]
// Implemented by `src/source_toolchain_closure.rs`, `src/cargo_free_self_build.rs`,
// `src/rust_plan.rs`, and `src/main.rs`.

// r[verify rust_package_planning.source_built_toolchain_closure]
// Verified by archived evidence in
// `cairn/archive/2026-05-31-source-built-toolchain-closure/evidence/` and focused
// unit tests under the root package modules named above.

// Build-tool boundary first-batch bridge.
//
// r[impl build_tool_boundary.mantle_not_module_layer]
// Mantle-side implementation is the frontend-neutral build boundary in the root
// CLI/build path plus removal of the old `mantle system` surface. Durable design
// evidence lives in `adr/0010-keep-mantle-build-tool-boundary.md` and archived
// Cairn evidence under `cairn/archive/2026-05-31-onix-module-eval-boundary/`.
//
// r[verify build_tool_boundary.mantle_not_module_layer]
// Verified by `tests/removed_system_cli.rs`: build-shaped frontend inputs reach
// planning, raw module inventory is rejected, and removed module-layer surfaces
// stay absent from public and implementation scans.
//
// r[impl build_tool_boundary.onix_owns_module_lowering]
// Mantle implements only its side of this boundary: Onix module lowering remains
// outside Mantle core, and Mantle accepts build-shaped inputs after an external
// frontend has done module evaluation. This is traceability for the Mantle-side
// non-ownership rule; it does not claim Onix module lowering is implemented here.
//
// r[verify build_tool_boundary.onix_owns_module_lowering]
// Verified by `tests/removed_system_cli.rs` and the archived
// `onix-module-eval-boundary` evidence, which guard against raw Onix role/tag /
// provider semantics becoming Mantle build inputs or diagnostics.
//
// r[impl build_tool_boundary.synthetic_system_eval_not_integration]
// Mantle-side implementation quarantines the old synthetic system scaffold by
// removing it from supported CLI/docs/stdlib surfaces and by keeping production
// integration dependent on an external frontend lowering into build inputs.
//
// r[verify build_tool_boundary.synthetic_system_eval_not_integration]
// Verified by `tests/removed_system_cli.rs::system_eval_is_not_a_supported_subcommand`
// and the public/implementation surface scans in that test module.

// Examples support contract bridge.
//
// r[impl examples.support_catalog]
// Implemented by `examples/catalog.ncl`, which is evaluated as typed Nickel and
// consumed by `tests/examples_inventory.rs` to classify every checked-in
// user-facing example by support tier, capability, and validation rail.
//
// r[verify examples.support_catalog]
// Verified by `tests/examples_inventory.rs`: positive coverage parses the live
// catalog and checks every checked-in user-facing example path; negative tests
// reject duplicate ids/paths, unsupported support tiers, silent skips, and
// missing catalog coverage.
//
// r[impl examples.documentation_drift]
// Implemented by the catalog-backed documentation index in `examples/README.md`
// plus the root README examples section. The drift rail keeps generated and
// real-network examples explicitly classified instead of relying on prose only.
//
// r[verify examples.documentation_drift]
// Verified by `tests/examples_inventory.rs`: README drift tests reject omitted
// catalog paths, stale example links, and stale legacy branding outside exact
// compatibility identifiers such as `crunch.ncl`.
//
// r[impl examples.validation_matrix]
// Implemented by `tests/examples_eval.rs` and `tests/examples_build.rs`, which
// consume the catalog's eval rails, run fast build smoke tests in temp
// store/state roots, and keep heavyweight examples behind explicit ignored
// tests or capability skips.
//
// r[verify examples.validation_matrix]
// Verified by `tests/examples_eval.rs` catalog-driven eval coverage plus
// negative malformed/missing-seed assertions, and by `tests/examples_build.rs`
// fast/offline build smoke, missing-selector, and intentional-failure coverage.
//
// r[impl examples.output_execution]
// Implemented by `tests/examples_build.rs`, which inspects flat outputs,
// multi-output layouts, project check result files, and the preserved ignored
// crc64 binary execution test for heavyweight validation.
//
// r[verify examples.output_execution]
// Verified by `tests/examples_build.rs` assertions for hello/multi-step output
// content, local multi-output fixture files, project check result output, and
// fail-closed handling of diagnostic examples.
//
// r[impl examples.offline_fetcher_fixtures]
// Implemented by generated local file, tarball, and git fixtures in
// `tests/examples_build.rs`, and by catalog/docs rails that map real-network
// cookbook examples to their offline validation fixture families.
//
// r[verify examples.offline_fetcher_fixtures]
// Verified by `tests/examples_build.rs` offline fetchurl, fetchTarball, and
// fetchGit tests using temp store/state roots plus test-owned local sources, and
// by `tests/examples_inventory.rs` rejecting fetcher catalog entries without
// offline fixture rails.
//
// r[impl examples.fixed_output_negative_cases]
// Implemented by wrong-hash file, tarball, and git fixture tests plus a temp
// `--fix` repair workflow in `tests/examples_build.rs`, so checked-in examples
// are not mutated during hash repair validation.
//
// r[verify examples.fixed_output_negative_cases]
// Verified by `tests/examples_build.rs::offline_fetcher_wrong_hashes_fail_closed`
// and `fix_flag_updates_temp_fetchurl_fixture_hash`, which assert fixed-output
// mismatch diagnostics, no successful output reporting, empty failed temp stores,
// and a corrected temp fixture hash.
//
// r[impl examples.progressive_gallery]
// Implemented by the progressive lane order and command/output tables in
// `examples/README.md`, the local named-output example
// `examples/local-output-layout.ncl`, and project workflow documentation under
// `examples/project/README.md`.
//
// r[verify examples.progressive_gallery]
// Verified by `tests/examples_inventory.rs` progressive lane/project-doc checks,
// `tests/examples_eval.rs` catalog-driven evaluation, `tests/examples_build.rs`
// local output-layout assertions, and the module-boundary guard in
// `tests/removed_system_cli.rs`.
//
// r[impl examples.trust_provenance_gallery]
// Implemented by the trust/provenance lane in `examples/README.md`, which gives a
// runnable JSON build-report recipe and keeps the self-build skeleton documented
// as a non-claim rather than fake proof evidence.
//
// r[verify examples.trust_provenance_gallery]
// Verified by `tests/examples_build.rs::hello_json_build_report_exposes_artifact_attestation_shape`
// for deterministic local evidence shape, and by `tests/examples_inventory.rs`
// negative checks requiring artifact-attestation wording plus release/witness
// non-claim text.
//
// r[impl examples.resumable_remote_transfer_workflow]
// Implemented by the occurrence-safe demand core in
// `crates/crunch-build/src/distributed/remote_transfer.rs`, the production stdio
// composition in `src/remote_build.rs`, and the checked
// `examples/projects/remote-build-loopback` project/runbook.
//
// r[verify examples.resumable_remote_transfer_workflow]
// Verified by repeated-content positive/forged-offset core coverage, production
// interruption/resume and acknowledged-chunk tamper fixtures in
// `tests/remote_transfer_production.rs`, and catalog/non-claim drift checks in
// `tests/examples_inventory.rs`. Durable review evidence lives under
// `cairn/changes/publish-resumable-remote-transfer-workflow/evidence/` until
// archive.

// Compiled-eval legacy OpenSpec bridge.
//
// These references close Tracey linkage to the archived, legacy OpenSpec
// compiled-eval prototype work only. They do not claim a shipped compiled
// evaluator, full Nickel semantic support, or renewed priority for codegen work.
// Evidence mapping is recorded in the active Tracey change evidence file
// `compiled-eval-backfill-map-2026-06-01.md`.
//
// r[impl compiled-eval.backend-boundary]
// r[verify compiled-eval.backend-boundary]
// Implemented by the private `crunch-eval` backend seam in
// `crates/crunch-eval/src/backend.rs` and existing public helpers in
// `crates/crunch-eval/src/lib.rs`; verified by archived V1/V3 evidence.
//
// r[impl compiled-eval.backend-boundary.default]
// r[verify compiled-eval.backend-boundary.default]
// Current default helpers call `default_backend()` / `NickelBackend`, while the
// Cranelift path is gated behind `cranelift-proto`; archived tests cover default
// interpreter behavior.
//
// r[related compiled-eval.backend-boundary.swap]
// Future backend-swap compatibility is represented as a private-seam design
// constraint only; no future backend support is claimed by this bridge.
//
// r[related compiled-eval.profiling-gate]
// r[related compiled-eval.profiling-gate.no-evidence]
// r[related compiled-eval.profiling-gate.evidence]
// Archived V2/design evidence records profiling/benchmark gates and the current
// decision to keep compiled-eval tabled unless future data reopens it.
//
// r[related compiled-eval.benchmark-guardrail]
// r[related compiled-eval.benchmark-guardrail.recorded]
// r[related compiled-eval.benchmark-guardrail.regression]
// Archived benchmark evidence records the baseline bundle and a non-eval
// guardrail regression, so the prototype stayed non-shipping.
//
// r[related compiled-eval.cranelift-first]
// r[related compiled-eval.cranelift-first.initial]
// r[related compiled-eval.cranelift-first.llvm-later]
// Archived design records Cranelift as the first experiment and keeps LLVM as
// optional future work requiring a later measured reason.
//
// r[impl compiled-eval.private-backend-seam]
// r[verify compiled-eval.private-backend-seam]
// r[impl compiled-eval.private-backend-seam.callers]
// r[verify compiled-eval.private-backend-seam.callers]
// r[impl compiled-eval.private-backend-seam.no-leak]
// r[verify compiled-eval.private-backend-seam.no-leak]
// `EvalBackend`, `EvalRequest`, and Cranelift request handling stay private to
// `crunch-eval`; existing public helper signatures remain interpreter-shaped.

// Tracey coverage readiness bridge.
//
// r[impl verification_evidence.tracey_coverage_readiness]
// Implemented by the bounded backfill workflow recorded in
// `cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/`, plus this
// bridge file that keeps accepted requirements linked while root-package source
// scanning remains limited.
//
// r[verify verification_evidence.tracey_coverage_readiness]
// Verified by archived evidence in
// `cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/`, which
// records baseline counts, grouped missing IDs, first-batch refs, validation,
// tasks gates, and explicit non-claim status for remaining coverage debt.
//
// r[impl verification_evidence.release_witness_rebuild_multi_output]
// Implemented by `src/witness_rebuild.rs`: release witness rebuild plans derive
// every expected published output from the release manifest plus signed release
// attestation, then bind self-hosting proof-bundle artifacts and validated
// workflow-produced provider fixed-point stage binaries to those outputs by
// BLAKE3 digest before witness sidecar creation.
//
// r[verify verification_evidence.release_witness_rebuild_multi_output]
// Verified by `src/witness_rebuild.rs` unit tests, `tests/release_cli.rs`
// witness-rebuild CLI/helper tests, archived evidence in
// `cairn/archive/2026-06-26-witness-rebuild-two-output-release/evidence/`, and
// pending provider-bound replay evidence under
// `cairn/changes/witness-provider-bound-replay/evidence/`.
//
// r[impl verification_evidence.global_reproducibility_claim_admission]
// r[impl verification_evidence.global_reproducibility_reports]
// Implemented by `crates/crunch-release-core/src/global_reproducibility.rs`
// pure admission/report evaluation plus the thin CLI shell in
// `src/global_reproducibility_cmd.rs`. `src/release_cmd.rs` keeps release
// verification explicitly `not-evaluated` for global reproducibility.
//
// r[verify verification_evidence.global_reproducibility_claim_admission]
// r[verify verification_evidence.global_reproducibility_reports]
// Verified by `crunch-release-core` positive/negative global reproducibility
// unit tests and `mantle` bin tests for loading evidence, writing canonical
// reports, and blocking missing global evidence.
//
// r[impl verification_evidence.global_reproducibility_release_surface_evidence]
// r[verify verification_evidence.global_reproducibility_release_surface_evidence]
// Implemented by `src/global_reproducibility_release.rs`, which derives release
// bundle facts into surface evidence while leaving final admission to
// `src/global_reproducibility_cmd.rs`. Verified by positive stage2/full-release
// helper tests, invalid-provider blocker tests, and provider proof copied-bundle
// verifier tests in `src/cargo_free_self_build.rs`.
//
// r[impl verification_evidence.provider_fixed_point_path_normalization]
// r[verify verification_evidence.provider_fixed_point_path_normalization]
// Implemented by deterministic release path mode and provider fixed-point proof
// metadata generation in `src/cargo_free_self_build.rs`; verified by focused
// path-normalization, bundle-local metadata, and copied-bundle verifier tests.
//
// r[impl compiled-eval.cranelift-prototype-subset]
// r[verify compiled-eval.cranelift-prototype-subset]
// r[impl compiled-eval.cranelift-prototype-subset.default]
// r[verify compiled-eval.cranelift-prototype-subset.default]
// r[impl compiled-eval.cranelift-prototype-subset.unsupported]
// r[verify compiled-eval.cranelift-prototype-subset.unsupported]
// The prototype is feature-gated, default-off, limited to flat derivation
// fields, and rejects unsupported fields/import paths; archived V1/V4 evidence
// records the focused tests and source audit.
//
// r[related compiled-eval.future-semantics]
// r[related compiled-eval.future-semantics.contracts]
// r[related compiled-eval.future-semantics.shape]
// Future semantic-equivalence requirements remain non-claims: the current guard
// is that unsupported broader semantics stay outside the prototype and the
// interpreter remains the reference path.

// Nix-like build correctness primitive bridge.
//
// r[impl build_correctness.action_spec]
// r[verify build_correctness.action_spec]
// Implemented by `src/build_correctness.rs` canonical `mantle-action-spec-v1`
// records and verified by `build_correctness_action_spec_*` unit tests.
//
// r[impl build_correctness.nickel_eval_source_closure]
// r[verify build_correctness.nickel_eval_source_closure]
// Implemented by `src/build_correctness.rs` Nickel evaluation receipt DTOs and
// undeclared-import validation; verified by the Nickel eval receipt unit test.
//
// r[impl build_correctness.cas_object_store]
// r[verify build_correctness.cas_object_store]
// Implemented by `src/build_correctness.rs` CAS object manifests for files,
// directories, symlinks, generated payloads, redacted secret descriptors, and
// path-view-only rejection; verified by CAS positive/negative unit tests.
//
// r[impl build_correctness.hermetic_execution_policy]
// r[verify build_correctness.hermetic_execution_policy]
// Implemented by `src/build_correctness.rs` sandbox/network policy validation
// and enforced/unsupported sandbox reports; verified by hermetic policy tests.
//
// r[impl build_correctness.output_reference_scanning]
// r[verify build_correctness.output_reference_scanning]
// Implemented by `src/build_correctness.rs` reference scan reports and
// fail-closed diagnostics for undeclared, forbidden, traversal, duplicate-view,
// and plaintext-secret findings; verified by reference scan tests.
//
// r[impl build_correctness.reuse_and_substitution]
// r[verify build_correctness.reuse_and_substitution]
// Implemented by `src/build_correctness.rs` receipt-equivalence reuse admission
// over action refs, object refs, policies, producers, and signatures; verified
// by reuse admission tests.
//
// r[impl build_tool_boundary.correctness_primitives_frontend_neutral]
// r[verify build_tool_boundary.correctness_primitives_frontend_neutral]
// Implemented by treating frontend spec refs as opaque data in
// `src/build_correctness.rs`; verified by the frontend-boundary unit test.
//
// r[impl verification_evidence.build_correctness_receipts]
// r[verify verification_evidence.build_correctness_receipts]
// Implemented by `src/build_correctness.rs` deterministic
// `mantle-action-receipt-v1` receipts plus bounded JSON rendering; verified by
// receipt determinism/non-claim tests and archived change evidence.

// Project input retention bridge.
//
// r[impl project_workflows.input_retention_roots]
// Implemented by `crates/crunch-project-core/src/retention.rs` pure policy and
// root-action planning plus root-state shell persistence in `src/project_cmd.rs`.
//
// r[verify project_workflows.input_retention_roots]
// Verified by `crates/crunch-project-core/src/retention.rs` positive and
// negative retention tests, `tests/project_cli.rs` retention diagnostics tests,
// and archived evidence under
// `cairn/archive/2026-07-01-project-input-retention-roots/evidence/`.
//
// r[impl project_workflows.input_retention_atomicity]
// Implemented by same-directory temporary commits for `.mantle/retention.json`,
// root marker persistence under `.mantle/retention-roots/`, and uncommitted
// retention-state quarantine in the project CLI shell.
//
// r[verify project_workflows.input_retention_atomicity]
// Verified by retention core interruption tests, CLI shell tests for atomic root
// persistence, and the archived project-input-retention-roots validation
// transcript.

// Release bundle tree-copy confinement bridge.
//
// r[impl mantle.release_provenance.bundle_tree_copy.plan]
// r[impl mantle.release_provenance.bundle_tree_copy.plan.invalid]
// The pure deterministic planner and named entry/depth/path bounds live in
// `crates/crunch-release-core/src/tree_copy.rs`.
//
// r[impl mantle.release_provenance.bundle_tree_copy.no_follow]
// r[impl mantle.release_provenance.bundle_tree_copy.destination_confinement]
// r[impl mantle.release_provenance.bundle_tree_copy.symlink_policy]
// The no-follow observation, capability-confined execution, source revalidation,
// and hash shell live in `src/release_tree_copy.rs`; release creation preflights
// its directory plans in `src/release_evidence.rs` before bundle mutation.
//
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.positive]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
// r[verify mantle.release_provenance.bundle_tree_copy.validation]
// r[verify mantle.release_provenance.bundle_tree_copy.validation.production]
// Verified by positive planner/copy/hash fixtures and adversarial source,
// destination, target, type-drift, special-file, bound, and external-sentinel
// tests in `crates/crunch-release-core/src/tree_copy.rs`,
// `src/release_tree_copy.rs`, and `src/release_evidence.rs`.

// Release verification terminal-decision bridge.
//
// r[impl mantle.operator_diagnostics.release_verification.terminal_verdict]
// r[impl mantle.operator_diagnostics.release_verification.json_contract]
// r[impl mantle.operator_diagnostics.release_verification.render_boundary]
// Implemented by `src/release_cmd.rs`: the shell collects release facts, calls
// the fixed pure decision core, renders one completed immutable decision, and
// emits the human success marker only for final acceptance.
//
// r[verify mantle.operator_diagnostics.release_verification.fixtures.positive]
// r[verify mantle.operator_diagnostics.release_verification.fixtures.negative]
// r[verify mantle.operator_diagnostics.release_verification.json_negative]
// r[verify mantle.operator_diagnostics.release_verification.render_boundary.test]
// Verified by positive/negative release CLI fixtures in `tests/release_cli.rs`
// plus the root-package pure renderer test in `src/release_cmd.rs`.
// Pinned ast-grep structural evidence bridge.
//
// r[impl mantle.ast_grep_structural_rails.toolchain]
// The explicit ast-grep version pin, packaged executable, generated BLAKE3
// identity record, development-shell profile, and identity smoke live in
// `flake.nix`.
//
// r[verify mantle.ast_grep_structural_rails.toolchain]
// Verified by `checks.<system>.ast-grep-package-identity`, which recomputes the
// packaged executable BLAKE3 and checks the reported package/version identity.
//
// r[impl mantle.ast_grep_structural_rails.shell_boundary]
// Capability-confined no-follow reads and raw sidecar hashing live in
// `src/ast_grep_evidence.rs`; the no-std validator lives in
// `crates/crunch-release-core/src/ast_grep.rs`.
// Mantle does not automatically invoke ast-grep.
//
// r[verify mantle.ast_grep_structural_rails.shell_boundary]
// Verified by the shell-adapter boundary test and the no-std core build.
//
// r[impl mantle.ast_grep_structural_rails.validation]
// r[verify mantle.ast_grep_structural_rails.validation]
// Verified by positive and negative fixture tests, focused build-report and
// release-attachment tests, the package identity smoke, and Cairn lifecycle
// validation/gates recorded for the active change.
// Durable remote-attempt fencing bridge.
//
// r[impl remote_builds.durable_attempt_fencing]
// r[verify remote_builds.durable_attempt_fencing]
// The pure identity/fence model lives in
// `crates/crunch-build/src/distributed/remote_attempt.rs`; the durable shell is
// in `src/remote_build.rs`. Focused coordinator tests cover persistence-before-
// exposure, fresh assignment nonces, fail-closed legacy migration, state-reset
// identity separation, restart/reassignment, and stale report rejection across
// start, log, transfer, result, failure, and completion events.
//
// r[impl remote_builds.idempotent_attempt_reporting]
// r[verify remote_builds.idempotent_attempt_reporting]
// Canonical BLAKE3 payload digests and bounded event retention are implemented
// in the pure attempt core. Coordinator tests verify identical redelivery leaves
// durable bytes unchanged, finished-undelivered admission is reconstructed only
// after cryptographic revalidation, and conflicting event reuse changes no
// mutable surface.
//
// r[impl remote_builds.pure_attempt_decisions]
// r[verify remote_builds.pure_attempt_decisions]
// Authorization, retry, transition, idempotency, and fence decisions consume
// immutable facts in the `crunch-build` core. Table, proptest, and Kani harnesses
// cover deterministic decisions, monotonic fences, and terminal-state closure.

// Machine artifact contract registry bridge.
//
// r[impl mantle.machine_artifact_contracts.inventory]
// r[verify mantle.machine_artifact_contracts.inventory]
// The typed registry and producer annotations classify public JSON families;
// checker tests reject duplicate IDs, stale markers, and unclassified root JSON.
//
// r[impl mantle.machine_artifact_contracts.authority]
// r[verify mantle.machine_artifact_contracts.authority]
// Rust DTO snapshots own emitted facts; generated eager Nickel predicates and
// Rust-serialized fixtures provide review/test authority without runtime Nickel.
//
// r[impl mantle.machine_artifact_contracts.registry_rail]
// r[verify mantle.machine_artifact_contracts.registry_rail]
// Pure modules under `scripts/machine_schema_contracts/` feed the thin checker
// shell; mutation-free self-tests cover positive and negative rail behavior.
//
// r[impl mantle.machine_artifact_contracts.contract_vocabulary]
// r[verify mantle.machine_artifact_contracts.contract_vocabulary]
// The shared prelude and deterministic renderer enforce exact versions, closed
// shapes, bounds, digests, references, redaction, and cross-field invariants.
//
// r[impl mantle.machine_artifact_contracts.freshness]
// r[verify mantle.machine_artifact_contracts.freshness]
// BLAKE3 binds owner identity, source bytes, schemas, generated contracts,
// prelude, fixtures, consumer policy, version policy, and non-claims.
//
// r[impl mantle.machine_artifact_contracts.initial_cohort]
// r[verify mantle.machine_artifact_contracts.initial_cohort]
// Thirteen contracted build, plan, route, receipt, source-bundle, Nickel-export,
// doctor, and release-envelope surfaces carry schemas and generated contracts.
//
// r[impl mantle.machine_artifact_contracts.fixtures]
// r[verify mantle.machine_artifact_contracts.fixtures]
// Every contracted surface has positive fixtures and categorized negative sets;
// producer parity and embedded Nickel tests exercise both directions.
//
// r[impl mantle.machine_artifact_contracts.versioning]
// r[verify mantle.machine_artifact_contracts.versioning]
// Registry validation rejects undeclared compatibility; no prior versions are
// supported without an explicit converter and migration fixtures.
//
// r[impl mantle.machine_artifact_contracts.runtime_boundary]
// r[verify mantle.machine_artifact_contracts.runtime_boundary]
// Product sources contain no machine-contract Nickel execution path; source
// guards and producer tests keep validation confined to review/test tooling.

// Remote-attempt observability bridge.
//
// r[impl remote_builds.immutable_attempt_log_segments]
// r[verify remote_builds.immutable_attempt_log_segments]
// r[impl remote_builds.pure_log_cursor_kernel]
// r[verify remote_builds.pure_log_cursor_kernel]
// Immutable segment/manifest/anchor decisions live in `crunch-build`; the
// no-follow durable shell and coordinator summary integration live under
// `src/{remote_attempt_log_store,remote_build}.rs`.
//
// r[impl remote_builds.diagnostic_trace_context]
// r[verify remote_builds.diagnostic_trace_context]
// Bounded W3C parsing and digest-only health live in
// `src/remote_trace_context.rs`; production stdio tests prove propagation,
// malformed dropping, and authority invariance.
//
// r[impl operator_diagnostics.remote_execution_telemetry]
// r[verify operator_diagnostics.remote_execution_telemetry]
// r[impl operator_diagnostics.telemetry_exporter_isolation]
// r[verify operator_diagnostics.telemetry_exporter_isolation]
// Canonical events/metric admission live in `crunch-build`; production lifecycle
// wiring, disabled-by-default Prometheus/OTLP shells, immutable summaries, and
// build-report health live under root `src/`. Scheduler-priority instrumentation
// remains an explicit active-task blocker until a real lazy-goal priority
// decision reaches the remote dispatch seam.

// Remote failure-debug bundle bridge.
//
// r[impl operator_diagnostics.remote_failure_debug_bundle]
// r[verify operator_diagnostics.remote_failure_debug_bundle]
// r[impl operator_diagnostics.remote_failure_replay]
// r[verify operator_diagnostics.remote_failure_replay]
// r[impl remote_builds.failure_debug_capture]
// r[verify remote_builds.failure_debug_capture]
// Deterministic bundle/capture/replay/comparison/retention kernels live in
// `crates/crunch-build/src/distributed/remote_failure_debug.rs`. The root shell
// owns atomic bundle publication, immutable-log reference composition,
// capability-confined no-follow capture, worker quarantine cleanup, redacted
// inspect/replay-plan/GC rendering, newly identified replay jobs, ordinary
// fenced output admission, and bounded status reports. Core, shell,
// multiprocess publication, worker capture, clean-process inspect, retention,
// and failed/successful replay tests provide positive and negative evidence.

// External batch dispatcher bridge.
//
// r[impl external_batch_dispatchers.protocol]
// r[verify external_batch_dispatchers.protocol]
// r[impl external_batch_dispatchers.canonical_identity]
// r[verify external_batch_dispatchers.canonical_identity]
// r[impl external_batch_dispatchers.typed_configuration]
// r[verify external_batch_dispatchers.typed_configuration]
// r[impl external_batch_dispatchers.resources]
// r[verify external_batch_dispatchers.resources]
// r[impl external_batch_dispatchers.fenced_lifecycle]
// r[verify external_batch_dispatchers.fenced_lifecycle]
// r[impl external_batch_dispatchers.adapter_confinement]
// r[verify external_batch_dispatchers.adapter_confinement]
// r[impl external_batch_dispatchers.worker_handoff]
// r[verify external_batch_dispatchers.worker_handoff]
// r[impl external_batch_dispatchers.slurm_adapter]
// r[verify external_batch_dispatchers.slurm_adapter]
// r[impl external_batch_dispatchers.diagnostics]
// r[verify external_batch_dispatchers.diagnostics]
// r[impl external_batch_dispatchers.hardware_workload_composition]
// r[verify external_batch_dispatchers.hardware_workload_composition]
// r[impl external_batch_dispatchers.final_validation]
// r[verify external_batch_dispatchers.final_validation]
// Provider-neutral protocol and reconciliation logic live in
// `crates/crunch-build/src/distributed/external_batch.rs`; typed configuration,
// confined direct/Slurm process execution, fenced coordinator persistence,
// worker registration, ordinary CAS transfer, output admission, and bounded
// diagnostics live under root `src/`. Focused positive and negative tests cover
// identities, resource projection, restart, stale fences, duplicate locators,
// registration ordering, exact fake Slurm commands, malformed/flood/timeout
// failures, CAS import, signed output admission, and scheduler non-authority.
// Hardware composition remains intentionally unclaimed until the prerequisite
// `prove-hardware-simulation-build-flow` change provides that lane.
