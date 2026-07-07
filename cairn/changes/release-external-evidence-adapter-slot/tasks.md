## Phase 1: Manifest and core validation

- [ ] [serial] r[mantle.release_external_evidence] Add an opaque `ExternalEvidence` record and optional `external_evidence[]` manifest field without introducing Valence, Octet, Trellis, or Cairn crate/schema dependencies.
- [ ] [serial] r[mantle.release_external_evidence.validation] Implement pure validation for role, schema, bundle-local path, BLAKE3 digest, claim scope, and non-empty non-claims.
- [ ] [parallel] r[mantle.release_external_evidence.validation] Add positive tests for manifests with no external evidence and with a valid external-evidence sidecar.
- [ ] [parallel] r[mantle.release_external_evidence.validation] Add negative tests for path escape, missing sidecar, digest mismatch, empty role, empty schema, empty claim scope, and empty non-claims.

## Phase 2: CLI shell

- [ ] [serial] r[mantle.release_external_evidence.cli] Add release-create flags for explicitly bundling one or more external evidence sidecars with role, schema, and claim scope metadata.
- [ ] [serial] r[mantle.release_external_evidence.cli] Add release-verify role requirement flag for opt-in stack gates without changing default verification semantics.
- [ ] [parallel] r[mantle.release_external_evidence.cli] Add CLI tests for sidecar copy/digest recording and required-role failure.

## Phase 3: Documentation and validation

- [ ] [serial] r[mantle.release_external_evidence.boundary] Document that stack-specific sidecar semantics are verified by adapters such as Valence, not Mantle core.
- [ ] [serial] r[mantle.release_external_evidence] Run focused validation: `cargo test -p crunch-release-core`, release CLI tests covering external evidence, `cargo fmt --check`, and the relevant Cairn validation/gates.
