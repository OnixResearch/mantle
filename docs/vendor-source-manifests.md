# Vendor source manifests

Mantle vendor manifests describe vendored source identity and freshness. They do
not certify upstream quality, license compatibility, semantic correctness,
release eligibility, or build success unless separate evidence is linked.

## Contract

Each manifest row records:

- upstream repository identity;
- pinned upstream revision;
- filter definition and selected relative paths;
- expected BLAKE3 identities for selected vendored files;
- declared local edit paths;
- refresh command; and
- a visible `identity/freshness only` non-claim.

`vendor_source_manifest::validate_vendor_manifest` is the pure validation core.
It accepts already-loaded rows plus measured file identities and returns a
deterministic report. The shell helper `measure_vendor_files` owns filesystem
reads under an operator-provided root, rejects absolute paths and `../` escapes,
and computes BLAKE3 file identities for the pure core.

## Failure modes

Validation fails closed for stale digests, missing revision metadata, unsafe
paths, missing expected files, undeclared local edits under selected paths,
malformed BLAKE3 identities, duplicate measured paths, and missing non-claim
text.

## Stack reuse

UCAN-style vendor syncs can reuse the same row shape for Trellis or other
vendored source families: keep source identity, selected paths, local edits, and
refresh commands explicit, then attach separate license or semantic-review
evidence only when those stronger claims are required.
