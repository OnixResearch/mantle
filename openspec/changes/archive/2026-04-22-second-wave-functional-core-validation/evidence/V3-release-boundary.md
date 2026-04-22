Evidence-ID: second-wave-functional-core-validation-v3-release-boundary
Task-ID: V3
Artifact-Type: verification-note
Covers: architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.shell.adapters.effect.translation.release.evidence.bundle.io.in.shell
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

## Commands

```text
cargo test -p crunch-release-core --lib
cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip -- --nocapture
cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation -- --nocapture
cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch -- --nocapture
```

## Results

- `cargo test -p crunch-release-core --lib` → `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip -- --nocapture` → `test release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok`
- `cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact -- --nocapture` → `test release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok`
- `cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch -- --nocapture` → `test release_verify_rejects_manifest_schema_mismatch ... ok`
- `cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance -- --nocapture` → `test release_verify_rejects_missing_workflow_provenance ... ok`
- `cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation -- --nocapture` → `test release_verify_rejects_claim_boundary_violation ... ok`
- `cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch -- --nocapture` → `test release_verify_rejects_proof_linkage_source_digest_mismatch ... ok`

## Boundary inspection

Files inspected:

- `crates/crunch-release-core/src/lib.rs`
- `crates/crunch-release-core/src/manifest.rs`
- `crates/crunch-release-core/src/error.rs`
- `src/release_evidence.rs`
- `src/release_cmd.rs`

Observed boundary:

- `crunch-release-core` owns owned-data manifest/proof logic only:
  - `canonical_release_evidence_manifest(manifest: ReleaseEvidenceManifest)`
  - `validate_bundled_artifact_record(artifact: BundledArtifact, field_name: String)`
  - `extract_full_self_hosting_proof_identity_fields(manifest_bytes: Vec<u8>)`
- `src/release_evidence.rs` keeps std shell work:
  - filesystem reads and writes
  - bundle tree copying
  - directory hashing and NAR-style digest capture
  - proof-bundle loading from disk
  - mapping core errors into `RunError`
- `src/release_cmd.rs` keeps CLI-facing path resolution and operator output formatting.

## Required evidence summary

Round-trip release verification stays green. Malformed release evidence fails through the named negative tests. Inspection shows filesystem and CLI concerns remain outside `crunch-release-core`, while canonical manifest/proof-linkage validation stays in the core.
