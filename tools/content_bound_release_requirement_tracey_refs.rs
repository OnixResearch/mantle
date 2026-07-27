// Content-bound release-requirement Tracey bridge.
//
// Implementation and tests:
// - `crates/crunch-release-core/src/content_bound_requirements.rs`
// - `crates/crunch-release-core/src/manifest.rs`
// - `src/content_bound_requirement_evidence.rs`
// - `src/release_evidence.rs`
// - `src/release_cmd.rs`
// - `tests/release_cli.rs`
//
// Producer contract and operator guidance:
// - `fixtures/content-bound-requirements/integration-receipt.ncl`
// - `docs/content-bound-requirement-evidence.md`
//
// These references support lifecycle traceability only. Strict release policy
// uses measured content-bound rows. This bridge does not prove current file
// content, producer authority, requirement satisfaction, source correctness,
// test truth, runtime safety, or release eligibility.
//
// r[impl mantle.release_provenance.content_bound_requirement_coverage]
// r[verify mantle.release_provenance.content_bound_evidence_manifest]
// r[verify mantle.release_provenance.legacy_coverage_boundary]
