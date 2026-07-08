## Phase 1: Policy and bundle model

- [ ] [serial] r[mantle.release_provenance.valence_required_policy] Add a release profile policy field that declares stack-provenance sidecar evidence as optional or required.
- [ ] [serial] r[mantle.release_provenance.valence_receipt_binding] Add release bundle/report fields for the stack-provenance sidecar artifact, Valence verification receipt artifact, BLAKE3 hashes, role, schema, claim scope, binary identity, and non-claim boundary.
- [ ] [serial] r[mantle.release_provenance.opaque_boundary] Ensure Mantle reports the sidecar as Valence-validated external evidence without parsing or claiming Valence stack semantics.

## Phase 2: Verification and fixtures

- [ ] [parallel] r[mantle.release_provenance.fixture_matrix] Add positive fixtures for optional absent, optional present, and required present Valence-validated sidecar evidence.
- [ ] [parallel] r[mantle.release_provenance.fixture_matrix] Add negative fixtures for missing sidecar, wrong role, wrong schema, wrong claim scope, stale sidecar digest, missing Valence receipt, stale Valence receipt digest, missing binary identity, and weakened non-claims.
- [ ] [serial] r[mantle.release_provenance.valence_required_policy] Fail closed in required mode and emit a skipped/absent disposition in optional mode.

## Phase 3: Operator docs and validation

- [ ] [serial] r[mantle.release_provenance.opaque_boundary] Update release evidence docs and command help with the Valence-validated sidecar flow and non-claims.
- [ ] [serial] r[mantle.release_provenance.valence_receipt_binding] Run release evidence fixture tests, focused CLI tests, policy freshness checks, and Cairn validation/gates before archive.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Optional absent sidecar records skipped/absent disposition | Positive fixture test |
| Optional present sidecar verifies bundle-local metadata | Positive fixture test |
| Required present sidecar plus Valence receipt passes | Positive fixture test |
| Missing sidecar fails required mode | Negative fixture test |
| Wrong role/schema/claim scope fails required mode | Negative fixture test |
| Stale sidecar or receipt digest fails required mode | Negative fixture test |
| Missing binary identity fails required mode | Negative fixture test |
| Mantle opaque boundary is documented | Docs check or transcript |
