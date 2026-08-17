# Change: Prove bounded compatibility with an independent OCI registry

## Why

Mantle's registry transport is covered by a deterministic in-process server, but the accepted evidence still lacks an execution against an independently implemented OCI Distribution server. A pinned external implementation can reveal protocol assumptions without expanding the claim to arbitrary registry compatibility.

## What Changes

- Expose the repository-pinned `distribution` package as a reproducible compatibility fixture. r[kernel_bundle_oci.registry_external_compatibility]
- Repair incompatibilities found by that fixture: use OCI image-manifest schema 2 for ordinary and companion manifests, keep Mantle report schemas independent, and represent companion payloads as role-annotated layers over a canonical empty config. r[kernel_bundle_oci.registry_external_compatibility]
- Add an ignored production-CLI integration test that launches the independent registry process, performs signed push and fresh-state pull/admission, and rejects a wrong immutable signature-manifest digest without publishing output. r[kernel_bundle_oci.registry_external_compatibility]
- Add an operator command and durable lifecycle transcript identifying the exact implementation/version and bounded claim. r[kernel_bundle_oci.registry_external_compatibility]

## Impact

- Affected code: `flake.nix`, `src/oci_projection.rs`, `src/oci_projection/layout.rs`, `src/oci_registry.rs`, `src/oci_registry_shell.rs`, `tests/kernel_bundle_oci_registry_cli.rs`, and test support.
- Affected docs: OCI registry runbook and README validation commands.
- Companion wire encoding and OCI image/index schema versions change; receipt structures, trust policy, credentials, signed digest-pair semantics, and ordinary admission authority do not.
- Compatibility remains limited to the pinned local HTTP `distribution` fixture; authenticated deployment, redirects, proxies, TLS PKI, registry authorization, Referrers API, and arbitrary registry implementations remain non-claims.
