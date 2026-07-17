# Registry-backed OCI publication over the admitted local kernel-bundle fixture.
let Workflow = {
  schema | String,
  id | String,
  positive_commands | Array String,
  negative_cases | Array String,
  evidence_fields | Array String,
  non_claims | Array String,
} in

{
  schema = "mantle-example-workflow-v1",
  id = "kernel-bundle-oci-registry",
  positive_commands = [
    "mantle --json --state-dir $work/source-state artifact oci-push --layout $work/layout --registry $registry --repository onix/kernel-bundle --reference reviewed --bearer-token-file $token_file --receipt-out $work/push.json",
    "mantle --json --state-dir $work/pull-state artifact oci-pull --registry $registry --repository onix/kernel-bundle --reference reviewed --expected-manifest-digest $manifest_digest --expected-metadata-manifest-digest $metadata_manifest_digest --bearer-token-file $token_file --out $work/pulled-layout --report-out $work/import.json --receipt-out $work/pull.json",
  ],
  negative_cases = [
    "missing-or-denied-bearer-token",
    "mutable-image-tag-drift",
    "mutable-metadata-tag-drift",
    "tampered-metadata-or-descriptor-blob",
    "interrupted-image-manifest-publication",
    "existing-layout-or-receipt-destination",
  ],
  evidence_fields = [
    "manifest_digest",
    "metadata_manifest_digest",
    "layout_blake3",
    "projection_blake3",
    "uploaded_blobs",
    "reused_blobs",
    "expected_manifest_digest",
    "expected_metadata_manifest_digest",
    "import_receipt_blake3",
    "credential_mode",
    "non_claims",
  ],
  non_claims = [
    "no-registry-trust-claim",
    "no-authorization-proof",
    "no-tag-immutability-claim",
    "no-signature-verification",
    "no-exactly-once-publication",
    "no-upload-resumption",
    "no-kernel-compatibility-decision",
    "no-bootability-or-deployability-claim",
    "no-release-eligibility-claim",
    "bounded-distribution-subset-only",
  ],
} | Workflow
