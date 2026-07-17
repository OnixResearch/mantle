# Design: Independent OCI Distribution compatibility rail

## Goal

Prove that Mantle's production signed OCI push/pull path interoperates with one independent, pinned OCI Distribution implementation while preserving all existing trust and admission boundaries.

## Functional core and imperative shell

The external fixture first rejected the original wire shape: OCI artifact manifests were unsupported, and ordinary image/index documents incorrectly shared Mantle's report schema version 1. The pure core now emits and validates schema-version-2 OCI image manifests for ordinary and companion documents. Companion manifests preserve `artifactType`, `subject`, annotations, and role descriptors while using a canonical empty config plus `layers`. Mantle report schemas stay version 1 or 2 according to their own contracts. The test reuses the corrected pure manifest/signature/receipt validators and production CLI. A test-only shell owns the external child process, temporary filesystem, readiness polling, and bounded teardown.

## Fixture identity

`flake.nix` exports `pkgs.distribution` as `oci-distribution-registry`. The current repository pin resolves to OCI Distribution v3.1.0. The proof records normalized `registry --version` output without the local executable path. The fixture is independent of Mantle's in-process test server and is pinned by the repository's Nix input/lock.

## Execution

1. Resolve `MANTLE_TEST_DISTRIBUTION_REGISTRY` to an executable fixture.
2. Allocate a loopback endpoint with bounded retry, write a minimal filesystem-backed registry config, spawn `registry serve`, and wait for readiness within a named timeout.
3. Build the gallery OCI layout through public Mantle commands.
4. Push the layout with the typed Nickel trust policy and explicit signing key.
5. Pull by all three immutable manifest digests into fresh state and verify the contracted push/pull/import receipts plus exact layout bytes.
6. Repeat pull with a wrong signature-manifest digest and assert failure before output layout, import report, or success receipt publication.
7. Terminate and reap the external process within a bounded teardown path.

## Evidence

The test writes sanitized fixture-version, push, pull, and negative-path summaries when `MANTLE_EXTERNAL_REGISTRY_EVIDENCE_DIR` is set. Lifecycle evidence records the exact command and result. No credential material is required or emitted.

## Non-claims

The result does not establish arbitrary registry compatibility, authenticated-registry support, registry authorization, redirect/proxy behavior, TLS/PKI correctness, tag immutability, Referrers API support, transparency, revocation freshness, kernel correctness, bootability, deployability, or release eligibility.
