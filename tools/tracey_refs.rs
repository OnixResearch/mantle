// Mantle Tracey coverage bridge.
//
// Cairn's built-in tracey coverage rail currently scans `crates/` and `tools/`.
// Mantle's CLI/root-package implementation lives under top-level `src/`, so
// synced requirements implemented there need a small bridge until the coverage
// rail scans the package root directly.

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
// Export/import DTOs and redaction-safe writers exist, but registry linkage is
// intentionally deferred to active dependency
// `cairn/changes/expand-machine-artifact-contract-registry/`.

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
