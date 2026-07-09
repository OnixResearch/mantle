# Cairn release evidence handoff

Mantle can carry Cairn lifecycle and release-readiness evidence inside release
bundles as external evidence. Cairn owns lifecycle readiness, policy semantics,
change/archive consistency, and gate interpretation. Mantle owns only the
bundle-local handoff: artifact id, role, schema, BLAKE3 digest, Cairn policy
digest, release-readiness id, coverage ids, and non-claims.

`cairn_release_handoff::validate_cairn_release_evidence_handoff` is the pure
validator for already-loaded handoff rows. The release shell remains responsible
for reading files, computing BLAKE3 digests, and locating Cairn exports.

## Supported roles

- `cairn-release-readiness-receipt`
- `cairn-change-validation-receipt`
- `cairn-archive-evidence-index`

Every row must include a matching supported schema and state that Mantle is not
proving release correctness. Passing validation proves only that a Cairn evidence
handoff row is complete, supported, digest-shaped, and non-overclaiming. It does
not prove release correctness, build correctness, source correctness, artifact
correctness, or deployment safety.
