# Nominal trust boundaries

Mantle keeps structural wire records separate from admitted core values.

Each admitted value has a private representation and a fallible constructor. Wire DTOs keep their accepted field names and scalar spellings. Compatibility adapters project admitted values back to those DTOs.

## Admitted scopes

### Bootstrap and StageX

`crunch-bootstrap-core` admits distinct stage, artifact, executable-authorization, executable-role, lineage, and environment IDs.

It also admits absolute executable paths, timeout milliseconds, output-byte limits, parallel-job limits, and role-specific BLAKE3 values.

Lineage admission resolves graph endpoints to seed, source, generated-artifact, tool, or patch roles. Tool source lists retain their declared source role, including external sources not declared as local graph nodes.

### Remote protocol

The remote shell admits request, protocol-session, endpoint, and output IDs before protocol decisions use them.

Credential types remain owned by `remote_credentials`. They include ticket IDs, verifier IDs, validity windows, use limits, build-time limits, and upload-byte limits.

### Content-bound release evidence

`crunch-release-core` admits repository, requirement, release, specification-path, evidence-path, and lowercase BLAKE3 values.

Requirement and content-bound digests keep their domain enum and checked BLAKE3 value in one aggregate. A domain mismatch remains a separate deterministic issue.

### Frontend artifacts

Frontend admission uses distinct spec, validator, artifact, target, build-root, and spec-hash roles. The attestation is a wire projection of admitted values.

### Fetch and content-addressed planning

`crunch-build` admits fetch URLs, Git revisions, archive inputs, output directories, derivation names, output names, logical prefixes, and provisional or final store paths.

Stable positional APIs remain narrow compatibility wrappers. Their decision logic delegates to typed requests or typed values.

## Local claims

These types prove only local syntax, bounds, role separation, and selected relationships for supplied values.

They do not prove:

- filesystem presence or path authority
- artifact correctness or source trust
- remote peer trust
- safe I/O or sandbox enforcement
- compiler correctness
- evidence truth
- release eligibility
