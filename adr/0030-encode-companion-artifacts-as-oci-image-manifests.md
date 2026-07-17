# ADR 0030: Encode companion artifacts as OCI image manifests

## Status

Accepted (2026-07-17)

## Context

ADRs 0028 and 0029 establish subject-bound metadata and signature artifacts, but their first implementation encoded those documents with `application/vnd.oci.artifact.manifest.v1+json`. Mantle's deterministic in-process registry accepted that shape. The independent OCI Distribution v3.1.0 compatibility rail rejected it with `manifest invalid: unsupported manifest media type and no default available`.

The same external rail then exposed that Mantle's ordinary image and index documents used `schemaVersion: 1`; OCI image/index manifests require schema version 2. Local validation had shared the report schema version with OCI document schema version and therefore failed to detect the interoperability defect.

## Decision Drivers

- Preserve the exact subject, annotation, layer-role, digest-pair, and signature-policy semantics from ADRs 0028 and 0029.
- Use a manifest shape accepted by a pinned independent OCI Distribution implementation.
- Keep ordinary image/index documents conformant without changing Mantle report schema versions.
- Avoid registry-specific fallbacks or downgrade behavior.
- Keep compatibility claims bounded to executed evidence.

## Decision

Mantle encodes both companion artifacts as OCI image manifests with media type `application/vnd.oci.image.manifest.v1+json`, schema version 2, explicit `artifactType`, the existing image-manifest `subject`, the existing bounded annotations, a canonical `{}` config blob with media type `application/vnd.oci.empty.v1+json`, and the prior role-annotated payload descriptors in `layers`.

The empty config descriptor is part of the uploaded/downloaded closure and is verified exactly. Metadata reconstruction reads only the role-annotated metadata layers. Signature verification validates the exact empty config descriptor and reads the single role-annotated signature-document layer before content admission.

Mantle also separates `OCI_SCHEMA_VERSION = 2` from its version-1 export/import report schemas. OCI image manifests and indexes use the OCI constant; Mantle receipts retain their existing schema versions.

## Alternatives Considered

### Keep the OCI artifact manifest and waive Distribution compatibility

Rejected because the operator goal is an executable workflow against a real independent registry, and the unsupported media type blocks publication before any Mantle trust logic can run.

### Add a registry-specific retry or media-type downgrade

Rejected because success would depend on mutable server behavior, would complicate immutable digest handoff, and could make pull accept a shape different from the signed/published one.

### Put metadata in the ordinary image manifest

Rejected because it mutates the already-admitted image identity and collapses the separation between portable OCI content and exact Mantle reconstruction evidence.

### Use a non-empty image config carrying metadata

Rejected because metadata already has explicit role-annotated layers. A canonical empty config minimizes duplicate semantics while satisfying the image-manifest contract.

## Consequences

- Companion-manifest SHA-256 values and all signatures over those values change from the pre-compatibility implementation.
- Existing v2 push/pull receipts remain structurally compatible but must carry the new immutable digests.
- Registry accounting includes the canonical empty config blob, deduplicated by digest.
- Local OCI export identities change once because ordinary image/index schema versions become conformant.
- The independent Distribution v3.1.0 rail now exercises signed push, immutable-digest pull, exact reconstruction, and ordinary admission.
- This does not establish arbitrary registry compatibility, authenticated deployment, registry authorization, redirect/proxy behavior, TLS/PKI correctness, Referrers API support, tag immutability, artifact correctness, bootability, deployability, or release eligibility.
