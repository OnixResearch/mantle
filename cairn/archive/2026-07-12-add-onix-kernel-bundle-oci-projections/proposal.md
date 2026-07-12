## Why

OnixOS needs KBI-compatible OCI transport for independently versioned kernel bundles and add-on packs, while Mantle already owns frontend-neutral artifact admission, BLAKE3 CAS identity, deterministic materialization, and import/export receipts. A direct Onix-specific implementation in Mantle would violate the build-tool boundary; a plain directory export would lose OCI media types, SHA-256 descriptors, annotations, and round-trip structure.

Mantle should extend its generic admitted-artifact boundary with a typed OCI projection plan. Onix supplies and attests the semantic projection; Mantle verifies object identities, constructs or reads deterministic OCI layouts, preserves KBI-compatible media types and annotations as data, and returns transport evidence without deciding kernel compatibility or deployability.

## What Changes

- Add a generic versioned OCI projection DTO for admitted frontend artifacts, layers, config, annotations, safe CAS refs, and expected digest roles. r[kernel_bundle_oci.projection]
- Require successful frontend spec admission and exact object availability before OCI export or import promotion. r[kernel_bundle_oci.admission]
- Build deterministic OCI image layouts and directory layers with canonical metadata and no hidden host-path or Nix fallback. r[kernel_bundle_oci.layering] r[kernel_bundle_oci.export]
- Preserve OCI/KBI SHA-256 values as external protocol identities while retaining Mantle BLAKE3 identities for bytes, plans, layouts, reports, and receipts. r[kernel_bundle_oci.digest_roles]
- Verify imported OCI layouts descriptor-by-descriptor, import exact bytes into Mantle CAS, and return a frontend-neutral reconstruction response for Onix canonicalization. r[kernel_bundle_oci.import]
- Register stable export/import reports with the machine-artifact contract registry and add positive/negative round-trip fixtures. r[kernel_bundle_oci.reports] r[kernel_bundle_oci.verification]

## Impact

- **CLI**: the existing `mantle artifact` boundary gains explicit OCI projection export/import behavior rather than an Onix-specific command family.
- **Core**: pure planning, canonicalization, digest-role validation, and import classification remain separate from filesystem/CAS shells.
- **Interoperability**: KBI layer media types and annotations are preserved as frontend-supplied data; Mantle does not interpret KBI compatibility semantics.
- **Transport**: the first supported surface is a local OCI image layout. Registry push/pull credentials and remote transport remain a separate source/store concern.
- **Claims**: passing proves admitted bytes were projected to or recovered from a valid bounded OCI layout with matching identities. It does not prove kernel compatibility, bootability, module/eBPF safety, signature trust, deployability, or release eligibility.
