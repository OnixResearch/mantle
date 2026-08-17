# Change: Publish registry-backed OCI workflow

## Why

Mantle can produce and re-admit an exact local OCI image layout, but operators cannot publish that proven layout through Mantle or recover its admitted projection from a registry. Requiring an external CLI would lose Mantle's digest-role checks, projection linkage, report identity, credential boundary, and fresh-state admission evidence.

The next bounded operator slice is an OCI Distribution-compatible push/pull path over the existing local layout contract. It must publish the ordinary image manifest and exact descriptor blobs, preserve the Mantle export report and canonical layout/index bytes through a subject-bound metadata artifact, resolve mutable tags to immutable SHA-256 before pull, and pass the reconstructed layout through the existing import admission path.

## What Changes

- Add a bounded OCI Distribution v2 transport shell with HTTPS-by-default endpoints, explicit local-HTTP opt-in, explicit bearer-token files, disabled redirects/proxies, named limits, and no external registry CLI fallback. r[kernel_bundle_oci.registry_transport]
- Publish exact local layouts by uploading missing descriptor/metadata blobs, publishing a subject-bound Mantle metadata manifest, and publishing the user-facing image tag last. r[kernel_bundle_oci.registry_transport]
- Pull only against operator-supplied expected image and metadata manifest SHA-256 values, verify both manifests and every blob, reconstruct exact local layout bytes atomically, and invoke ordinary OCI import into fresh Mantle state. r[kernel_bundle_oci.registry_admission]
- Emit contracted push/pull receipts binding registry target facts, OCI SHA-256 identities, Mantle BLAKE3 identities, transferred/reused byte accounting, import receipt linkage, credential mode without credential material, and explicit non-claims. r[kernel_bundle_oci.registry_receipts]
- Add deterministic in-process registry positive/negative fixtures for authenticated round trip, tag drift, tampered blobs, denied credentials, interrupted publication, retry/reuse, and no-report/no-admission failure behavior. r[kernel_bundle_oci.registry_verification]
- Add a supported networked gallery workflow only after the production CLI path and adversarial rails pass. r[kernel_bundle_oci.registry_verification]

## Impact

- **Public CLI:** adds `mantle artifact oci-push` and `mantle artifact oci-pull`.
- **Architecture:** adds pure registry planning/receipt logic plus a thin HTTP/filesystem shell; existing local projection/import remains authoritative for content admission.
- **Machine artifacts:** adds Rust-owned push/pull receipt schemas, generated Nickel contracts, positive/negative fixtures, and inventory entries.
- **Documentation/gallery:** adds a registry-backed workflow while preserving the local workflow and its narrower non-claims.
- **Security:** credentials are read only from an explicit bounded file, never serialized; redirects and ambient proxies are disabled; HTTP requires explicit opt-in.
- **Non-claims:** no registry trust, tag immutability, signature verification, authorization proof, exactly-once publication, upload resumption, garbage collection, kernel compatibility, bootability, deployability, or release eligibility claim.
