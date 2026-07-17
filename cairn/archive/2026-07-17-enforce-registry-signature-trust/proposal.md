## Why

Mantle registry transport currently proves immutable image/metadata digest linkage but explicitly does not authenticate who authorized that pair. A registry writer or stolen bearer credential can replace both tags and present another internally consistent pair. Operators need a bounded cryptographic trust decision before pulled bytes reach ordinary OCI admission.

## What Changes

- Publish a deterministic detached-signature OCI artifact that binds the immutable image-manifest and Mantle metadata-manifest SHA-256 pair under an explicit trust domain.
- Require a typed Nickel trust policy and explicit signing key inputs for registry publication.
- Require the immutable signature-manifest digest and the same typed trust policy for pull, and verify trusted non-revoked Ed25519 signatures before downloading the content closure or publishing/admitting a local layout.
- Extend contracted push/pull receipts, schemas, generated review contracts, gallery workflow, operator documentation, and negative fixtures with policy/signature evidence and precise non-claims.
- Preserve registry routing, credential material, credential paths, signing-key paths, and local paths outside signed content identity and receipt identity.

## Impact

- **Files**: `src/oci_registry.rs`, `src/oci_registry_shell.rs`, `src/artifact_cmd.rs`, `src/main.rs`, registry tests/support, machine-contract schemas/inventory, gallery workflow/docs, ADR 0029, and lifecycle evidence.
- **Testing**: baseline registry core/CLI tests; positive multi-key trusted round trip; unknown, revoked, malformed, wrong-domain, wrong-repository-policy, digest-substitution, signature-tag-drift, and pre-admission failure cases; machine-contract drift checks; first-party quality, dependency policy, Tiger Style, Cairn gates, and Tracey coverage.
