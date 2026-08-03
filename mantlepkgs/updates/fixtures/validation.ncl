let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let success = {
  status = "success",
  artifact_identity_blake3 = [digest],
  reason_codes = [],
} in
{
  schema = "mantlepkgs-update-validation-evidence-v1",
  candidate_version = "v1.3.0",
  candidate_source_identity_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  catalog = success,
  build_observations = success,
  validation_roots = success,
  impact_report = success,
}
