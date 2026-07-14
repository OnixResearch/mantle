# Cairn release evidence handoff

Mantle can carry Cairn lifecycle and release-readiness evidence inside release
bundles as external evidence. Cairn owns lifecycle readiness, policy semantics,
change/archive consistency, and gate interpretation. Mantle owns only the
bundle-local handoff: artifact id, role, schema, BLAKE3 digest, Cairn policy
digest, release-readiness id, coverage ids, non-claims, and the exact reviewed
archive receipt for Cairn's authenticated-input prerequisite.

`cairn_release_handoff::validate_cairn_release_evidence_handoff` is the pure
validator for already-loaded handoff rows and the typed archive dependency. The
release shell remains responsible for bounded no-follow file reads, BLAKE3
measurement, and locating explicit Cairn exports.

## Supported roles

- `cairn-release-readiness-receipt`
- `cairn-change-validation-receipt`
- `cairn-archive-evidence-index`

Every row must include a matching supported schema and state that Mantle is not
proving release correctness. Every handoff must also bind the pinned archived
`authenticate-stack-provenance-inputs` receipt and exact Cairn revision. Passing
validation proves only that the dependency and handoff rows are measured,
complete, supported, linked to the same bundle, and non-overclaiming. Mantle does
not independently re-run producer signatures and does not prove Cairn, release,
build, source, artifact, or deployment correctness.
