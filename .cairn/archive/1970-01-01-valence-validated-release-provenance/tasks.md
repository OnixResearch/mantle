## Phase 1: Policy and bundle model

- [x] [serial] r[mantle.release_provenance.valence_required_policy] Add a release profile policy field that declares stack-provenance sidecar evidence as optional or required.
  - Completed: `mantle release verify --stack-provenance optional|required` evaluates stack provenance with `absent`, `present`, or `invalid` disposition; required mode fails closed.
- [x] [serial] r[mantle.release_provenance.valence_receipt_binding] Add release bundle/report fields for the stack-provenance sidecar artifact, Valence verification receipt artifact, BLAKE3 hashes, role, schema, claim scope, binary identity, and non-claim boundary.
  - Completed: `ReleaseEvidenceManifest.stack_provenance` binds sidecar and Valence graph-report roles, schemas, BLAKE3 digests, bundle-relative paths, release-binary path/digest, claim scope, and non-claims.
- [x] [serial] r[mantle.release_provenance.opaque_boundary] Ensure Mantle reports the sidecar as Valence-validated external evidence without parsing or claiming Valence stack semantics.
  - Completed: the core validates only bundle-local path/digest/role/schema/claim-scope/binary identity/non-claims and emits the Valence-owned stack-semantics boundary in CLI and JSON verification output.

## Phase 2: Verification and fixtures

- [x] [parallel] r[mantle.release_provenance.fixture_matrix] Add positive fixtures for optional absent, optional present, and required present Valence-validated sidecar evidence.
  - Completed: `crunch-release-core` manifest tests cover optional absent, optional present, and required present evidence; `tests/release_cli.rs` covers required-present CLI verification.
- [x] [parallel] r[mantle.release_provenance.fixture_matrix] Add negative fixtures for missing sidecar, wrong role, wrong schema, wrong claim scope, stale sidecar digest, missing Valence receipt, stale Valence receipt digest, missing binary identity, and weakened non-claims.
  - Completed: `stack_provenance_policy_rejects_required_invalid_fixture_matrix` covers the full negative matrix with deterministic diagnostics; release bundle tests cover multi-binary selection failure.
- [x] [serial] r[mantle.release_provenance.valence_required_policy] Fail closed in required mode and emit a skipped/absent disposition in optional mode.
  - Completed: required mode rejects absent or invalid stack provenance, while optional absent verification is valid with `absent` disposition.

## Phase 3: Operator docs and validation

- [x] [serial] r[mantle.release_provenance.opaque_boundary] Update release evidence docs and command help with the Valence-validated sidecar flow and non-claims.
  - Completed: README release-evidence guidance and Clap help now document `--stack-provenance-*` create flags, `--stack-provenance optional|required`, multi-binary selection, and the opaque non-claim boundary.
- [x] [serial] r[mantle.release_provenance.valence_receipt_binding] Run release evidence fixture tests, focused CLI tests, policy freshness checks, and Cairn validation/gates before archive.
  - Evidence so far: `cargo test -p crunch-release-core manifest`, `cargo test -p mantle release_evidence`, `cargo test -p mantle --test release_cli stack_provenance`, and touched-file `rustfmt --check` passed. Cairn policy/gates run before archive.

## Verification Coverage

| Scenario | Evidence |
|---|---|
| Optional absent sidecar records skipped/absent disposition | `stack_provenance_policy_accepts_optional_absent_evidence` |
| Optional present sidecar verifies bundle-local metadata | `stack_provenance_policy_accepts_optional_present_evidence` |
| Required present sidecar plus Valence receipt passes | `stack_provenance_policy_accepts_required_valid_evidence`; `release_create_and_verify_stack_provenance_sidecar` |
| Missing sidecar fails required mode | `stack_provenance_policy_rejects_required_absent_evidence`; negative matrix |
| Wrong role/schema/claim scope fails required mode | `stack_provenance_policy_rejects_required_invalid_fixture_matrix` |
| Stale sidecar or receipt digest fails required mode | `validate_rejects_stack_provenance_digest_mismatch`; negative matrix |
| Missing binary identity fails required mode | negative matrix |
| Mantle opaque boundary is documented | README release-evidence section; `validate_rejects_stack_provenance_semantic_promotion` |
