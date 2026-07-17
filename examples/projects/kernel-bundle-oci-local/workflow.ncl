# Frontend-neutral local OCI projection workflow over admitted fixture objects.
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
  id = "kernel-bundle-oci-local",
  positive_commands = [
    "mantle --json --state-dir $work/source-state artifact import fixtures/vmlinuz --report-out $work/kernel-import.json",
    "mantle --json --state-dir $work/source-state artifact import fixtures/modules --report-out $work/modules-import.json",
    "mantle --json --state-dir $work/source-state artifact oci-export --projection fixtures/projection.json --spec-material fixtures/kernel-bundles-spec.md --source-admissions fixtures/source-admissions.json --out $work/layout",
    "mantle --json --state-dir $work/import-state artifact oci-import --layout $work/layout --report-out $work/import-report.json",
  ],
  negative_cases = [
    "unsealed-projection",
    "stale-source-admission",
    "tampered-layer-descriptor",
    "existing-output-layout",
  ],
  evidence_fields = [
    "projection_blake3",
    "layout_blake3",
    "layers[].blob_sha256",
    "objects[].artifact_ref",
    "round_trip",
    "non_claims",
  ],
  non_claims = [
    "no-registry-publication",
    "no-kernel-compatibility-decision",
    "no-bootability-claim",
    "no-deployability-claim",
    "no-release-eligibility-claim",
    "frontend-semantics-remain-external",
  ],
} | Workflow
