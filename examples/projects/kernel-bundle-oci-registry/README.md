# Registry-backed kernel-bundle OCI handoff

This supported networked workflow extends the checked local kernel-bundle OCI layout with Mantle's bounded OCI Distribution transport and explicit signature trust. It publishes the ordinary image manifest, subject-bound metadata artifact, and a detached-signature artifact over both immutable digests. Pull verifies the signed pair under a typed Nickel policy before downloading the content closure or invoking ordinary fresh-state OCI admission.

Start by following [`../kernel-bundle-oci-local/`](../kernel-bundle-oci-local/) through local export so `$work/layout` exists. Set an explicit registry origin and a bounded bearer-token file:

```sh
registry=https://registry.example.test
repository=onix/kernel-bundle
reference=reviewed
printf '%s\n' "$REGISTRY_BEARER_TOKEN" > "$work/registry-token"
chmod 0600 "$work/registry-token"
signing_key=/secure/operator/registry-signing-key
trust_policy=$PWD/examples/projects/kernel-bundle-oci-registry/trust-policy.ncl
mantle attest key-show --signing-key "$signing_key"
```

The checked-in policy contains only a demonstration public key. Its matching private key is absent.

For production, replace the trust domain, repository allowlist, trusted keys, required signers, threshold, and revocations.

`mantle attest key-show` prints the public verifier token for an existing key. It does not generate or mutate trust roots.

For a controlled local HTTP registry, use an origin such as `http://127.0.0.1:5000` and add `--allow-http` to both commands. Mantle rejects HTTP by default, does not read Docker configuration, does not consult ambient proxy variables, and does not follow redirects.

## Publish

```sh
mantle --json artifact oci-push \
  --layout "$work/layout" \
  --registry "$registry" \
  --repository "$repository" \
  --reference "$reference" \
  --trust-policy "$trust_policy" \
  --signing-key "$signing_key" \
  --bearer-token-file "$work/registry-token" \
  --receipt-out "$work/registry-push.json" \
  > "$work/registry-push-stdout.json"

manifest_digest=$(jq -r .manifest_digest "$work/registry-push.json")
metadata_manifest_digest=$(jq -r .metadata_manifest_digest "$work/registry-push.json")
signature_manifest_digest=$(jq -r .signature_manifest_digest "$work/registry-push.json")
```

Mantle uploads only absent blobs. It publishes `<reference>.mantle-metadata`, then `<reference>.mantle-signature`, and the user-facing image tag last.

The signature document domain-separates and signs the exact image and metadata SHA-256 pair. Mantle reads all three manifests by immutable digest before it writes the push receipt.

A failed attempt can leave unreferenced blobs or companion tags. The exact repeated command reuses verified content-addressed blobs.

This process does not prove a transaction, exactly-once publication, or resumable upload.

## Pull and admit into fresh state

```sh
mantle --json --state-dir "$work/registry-pull-state" \
  artifact oci-pull \
  --registry "$registry" \
  --repository "$repository" \
  --reference "$reference" \
  --expected-manifest-digest "$manifest_digest" \
  --expected-metadata-manifest-digest "$metadata_manifest_digest" \
  --expected-signature-manifest-digest "$signature_manifest_digest" \
  --trust-policy "$trust_policy" \
  --bearer-token-file "$work/registry-token" \
  --out "$work/registry-pulled-layout" \
  --report-out "$work/registry-import.json" \
  --receipt-out "$work/registry-pull.json" \
  > "$work/registry-pull-stdout.json"

jq '{published, manifest_digest, metadata_manifest_digest, signature_manifest_digest, trust_domain, policy_blake3, verified_signers, verified_public_key_blake3, layout_blake3, projection_blake3, uploaded_blobs, reused_blobs, credential_mode, non_claims}' \
  "$work/registry-push.json"
jq '{pulled, expected_manifest_digest, expected_metadata_manifest_digest, expected_signature_manifest_digest, trust_domain, policy_blake3, verified_signers, verified_public_key_blake3, layout_blake3, projection_blake3, import_state, import_receipt_blake3, credential_mode, non_claims}' \
  "$work/registry-pull.json"
```

All three expected manifest digests are mandatory. Pull resolves and checks the three tags, downloads only the bounded signature document, validates its subject/metadata/domain linkage, and enforces required signers, distinct-key threshold, and full-key revocations before downloading image or metadata content blobs. It rejects unknown/revoked keys, bad signatures, wrong repository/domain policy, companion drift, descriptor tampering, and stale import linkage without a receipt or admitted output. Success still requires `import_state: "admitted"` from the ordinary local OCI importer.

## Deterministic positive and negative rail

```sh
nix develop -c cargo test -p mantle --test kernel_bundle_oci_registry_cli -- --test-threads=1
nix develop -c cargo test -p mantle --lib oci_registry::tests -- --test-threads=1
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

The in-process loopback registry requires bearer authentication. It implements the endpoint subset that production `oci-push` and `oci-pull` use.

The tests check these cases:

- a trusted byte-identical round trip into fresh state
- tag drift and metadata-blob tampering
- unknown and revoked keys
- wrong repository policy and invalid signatures
- denied anonymous access
- interrupted publication without a receipt
- a repeated operation that reuses blobs without duplicate writes

## Non-claims

Successful verification authenticates only the immutable image/metadata digest pair under the supplied local policy. It does **not** establish registry authorization, transparency, revocation freshness, tag immutability, exactly-once publication, upload resumption, deletion/garbage collection, arbitrary registry compatibility, kernel compatibility, artifact correctness, bootability, deployability, or release eligibility. The default gallery server proves Mantle's reviewed OCI Distribution subset. The separate command in [`docs/kernel-bundle-oci.md`](../../../docs/kernel-bundle-oci.md#independent-registry-compatibility-rail) additionally proves the same production path against the repository-pinned OCI Distribution v3.1.0 process, but not another version or implementation. Credentials, key material, credential/key/policy paths, and registry routing remain outside signed content identity.
