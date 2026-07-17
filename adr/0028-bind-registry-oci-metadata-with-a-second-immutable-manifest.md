# ADR 0028: Bind registry OCI metadata with a second immutable manifest

## Status

Accepted (2026-07-16)

## Context

Mantle's admitted local OCI layout includes four classes of material: the ordinary image manifest and its descriptor blobs, `oci-layout`, `index.json`, and `mantle-oci-export-report.json`. OCI registries store manifests and blobs but do not preserve those local sidecar files automatically.

The export report carries the sealed projection and prior admission reductions needed for a pulled layout to regain `admitted` rather than `compatibility-only` state. Publishing only the image manifest loses that linkage. Mutating the image config or layers to embed the report would change the already-proven layout and external descriptor identities.

A registry tag is mutable. Requiring only the image-manifest digest does not prevent a registry writer from replacing an unsigned export report while retaining the same image bytes. BLAKE3 receipt identity is deterministic linkage, not authentication.

## Decision Drivers

- Publish the ordinary OCI image unchanged.
- Preserve exact local layout/index/export-report bytes.
- Prevent companion metadata substitution during an immutable operator handoff.
- Keep credentials and registry routing outside content identity.
- Avoid assuming Referrers API support in the bounded baseline.
- Preserve ordinary local OCI import as the only admission authority after pull.

## Decision

Mantle publishes a deterministic companion OCI artifact manifest under `<image-tag>.mantle-metadata`. Its subject is the exact ordinary image-manifest descriptor. Its three role-annotated blobs are the exact `oci-layout`, `index.json`, and `mantle-oci-export-report.json` bytes.

Push uploads or reuses all content-addressed blobs, publishes the companion manifest first, and publishes the user-facing image tag last. It re-reads both manifests by immutable digest before emitting a push receipt.

Pull requires two operator-supplied SHA-256 values from that push receipt: the expected image-manifest digest and expected metadata-manifest digest. It verifies both tag resolutions, the companion subject, all three metadata roles, every descriptor, and the exact reconstructed layout. The reconstructed layout then passes through ordinary OCI import and must return `admitted` before the registry pull receipt is emitted.

The companion tag is an explicit lookup convention, not a claim of Referrers API compatibility. Registry URL, repository, and tags are routing facts in registry receipts but do not alter OCI or Mantle content identities.

## Alternatives Considered

### Publish only the ordinary image

Rejected because exact Mantle projection/admission metadata would be lost and a pull could only recover generic compatibility evidence.

### Add the export report as an image layer or config field

Rejected because it would change the proven image manifest, descriptor graph, layout BLAKE3, and frontend expectations.

### Publish one tarball containing the local layout

Rejected because the primary registry object would no longer be the ordinary OCI image and external image consumers would need a Mantle-specific unpacking step.

### Discover the companion through the OCI Referrers API

Deferred because support is not uniform across the bounded target set. A future adapter may add referrers discovery while preserving the same subject and dual-digest invariants.

### Require only the image-manifest digest

Rejected because image identity does not authenticate unsigned companion metadata. Both immutable manifest digests are required.

## Consequences

- Registry push/pull preserves byte-identical local layout and admitted projection linkage without modifying the image.
- Push receipts become the handoff authority for both immutable manifest expectations.
- Failed pushes may leave unreferenced blobs or a companion tag; exact reruns can reuse blobs but are not transactions or resumable uploads.
- Credential files remain explicit shell inputs and are excluded from reports/content identity.
- Tests must cover both tag drifts, subject/role/blob tampering, denied credentials, interrupted image publication, and fresh-state admission.
- This decision does not prove registry trust, authorization, signatures, transparency, tag immutability, exactly-once publication, arbitrary registry compatibility, kernel compatibility, bootability, deployability, or release eligibility.
