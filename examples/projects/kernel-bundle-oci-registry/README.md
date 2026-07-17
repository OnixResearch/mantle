# Registry-backed kernel-bundle OCI handoff

This supported networked workflow extends the checked local kernel-bundle OCI layout with Mantle's bounded OCI Distribution transport. It publishes the ordinary image manifest plus a subject-bound companion metadata artifact, then pulls both immutable manifests and passes the byte-identical reconstructed layout through ordinary fresh-state OCI admission.

Start by following [`../kernel-bundle-oci-local/`](../kernel-bundle-oci-local/) through local export so `$work/layout` exists. Set an explicit registry origin and a bounded bearer-token file:

```sh
registry=https://registry.example.test
repository=onix/kernel-bundle
reference=reviewed
printf '%s\n' "$REGISTRY_BEARER_TOKEN" > "$work/registry-token"
chmod 0600 "$work/registry-token"
```

For a controlled local HTTP registry, use an origin such as `http://127.0.0.1:5000` and add `--allow-http` to both commands. Mantle rejects HTTP by default, does not read Docker configuration, does not consult ambient proxy variables, and does not follow redirects.

## Publish

```sh
mantle --json artifact oci-push \
  --layout "$work/layout" \
  --registry "$registry" \
  --repository "$repository" \
  --reference "$reference" \
  --bearer-token-file "$work/registry-token" \
  --receipt-out "$work/registry-push.json" \
  > "$work/registry-push-stdout.json"

manifest_digest=$(jq -r .manifest_digest "$work/registry-push.json")
metadata_manifest_digest=$(jq -r .metadata_manifest_digest "$work/registry-push.json")
```

Mantle uploads only absent blobs. It publishes `<reference>.mantle-metadata` first and the user-facing image tag last, then re-reads both manifests by SHA-256 before writing the push receipt. A failed attempt may leave unreferenced blobs or the companion tag; rerunning the exact command reuses verified content-addressed blobs. This is not a transaction, exactly-once publication, or resumable upload claim.

## Pull and admit into fresh state

```sh
mantle --json --state-dir "$work/registry-pull-state" \
  artifact oci-pull \
  --registry "$registry" \
  --repository "$repository" \
  --reference "$reference" \
  --expected-manifest-digest "$manifest_digest" \
  --expected-metadata-manifest-digest "$metadata_manifest_digest" \
  --bearer-token-file "$work/registry-token" \
  --out "$work/registry-pulled-layout" \
  --report-out "$work/registry-import.json" \
  --receipt-out "$work/registry-pull.json" \
  > "$work/registry-pull-stdout.json"

jq '{published, manifest_digest, metadata_manifest_digest, layout_blake3, projection_blake3, uploaded_blobs, reused_blobs, credential_mode, non_claims}' \
  "$work/registry-push.json"
jq '{pulled, expected_manifest_digest, expected_metadata_manifest_digest, layout_blake3, projection_blake3, import_state, import_receipt_blake3, credential_mode, non_claims}' \
  "$work/registry-pull.json"
```

Both expected digests are mandatory. The image digest alone cannot prevent a registry writer from substituting an unsigned companion export report. Pull rejects either tag drift, subject drift, missing or duplicate metadata roles, descriptor tampering, and stale import linkage before a successful receipt. Success requires `import_state: "admitted"` from the ordinary local OCI importer.

## Deterministic positive and negative rail

```sh
nix develop -c cargo test -p mantle --test kernel_bundle_oci_registry_cli -- --test-threads=1
nix develop -c cargo test -p mantle --lib oci_registry::tests -- --test-threads=1
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

The in-process loopback registry requires bearer authentication and implements the exact endpoint subset used by production `oci-push`/`oci-pull`. The tests prove authenticated byte-identical round trip into fresh state, image-tag drift rejection, metadata-blob tamper rejection, denied anonymous access, interrupted image publication without a receipt, and a rerun that reuses blobs without duplicate writes.

## Non-claims

This workflow does **not** establish registry trust, authorization policy, tag immutability, signature or transparency verification, exactly-once publication, upload resumption, deletion/garbage collection, arbitrary registry compatibility, kernel compatibility, bootability, deployability, or release eligibility. The deterministic server proves Mantle's reviewed OCI Distribution subset, not independent conformance of another registry implementation. Credentials and credential-file paths are excluded from reports and content identity.
