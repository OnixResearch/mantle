let digest_hex_length = 64 in
let git_oid_hex_length = 40 in
let Hex = fun length => std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value == length
  && std.string.is_match "^[0-9a-f]+$" value
) in
let Blake3 = Hex digest_hex_length in
let GitOid = Hex git_oid_hex_length in
let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^/[^\n]+$" value
) in
let root = "/home/brittonr/mantle-runs/dev-resume-20260904" in
let cache_path = "%{root}/dev-provider-cache-a7841aea" in
let LaterDev = {
  name | String,
  mode | [| 'resume, 'adopt |],
  predecessor | String,
  cache | AbsolutePath,
  requires_prior_report_review | Bool,
  requires_empty_output | Bool,
} in
{
  schema | String = "mantle-dev-resume-runtime-cohort-v1",
  status | String = "promoted-running",
  source_commit | GitOid = "a7841aea055bf643cc473164b98a2442014f080b",
  source_git_object_format | [| 'sha1 |] = 'sha1,
  source_path | AbsolutePath = "%{root}/source-a7841aea",
  source_record_blake3 | Blake3 = "48b14083b13dbc501009d5dd03e22c50f281592dd9f9f39f362f62fe1744f71b",
  vendor_record_blake3 | Blake3 = "2a34c833f27a7e2a6a509b466f414969d8b50ecfca8a1555c6845ddfc1298572",
  binary | AbsolutePath = "%{root}/mantle-a7841aea-release",
  binary_blake3 | Blake3 = "4819141a7c7452c1824cddf72b5f1b4d79965d5c6d5155e8feaa4e42dcc1ab6c",
  source_profile | AbsolutePath = "%{root}/authority/source-built-fixed-point-sources-a7841aea.json",
  source_profile_manifest_blake3 | Blake3 = "d507f1dd17104c43789b9a29df6656249027904ab3f161c3fee96870e4d10461",
  source_profile_file_blake3 | Blake3 = "6b643b28be9de586cc5cfd8aa2dccfce7238f72c3873545cf76fb03f2d111803",
  dev_plan_blake3 | Blake3 = "ddb30a9b18d0015110cde3cc69003403deeb71d8b706b803be38795f554ec6f7",
  stagex_lineage_blake3 | Blake3 = "e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d",
  native_provider_blake3 | Blake3 = "63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9",
  cold_attempt | String = "dev-cold-a7841aea",
  cold_mode | [| 'resume_enabled |] = 'resume_enabled,
  dev_cache | AbsolutePath = cache_path,
  report_bytes_max | Number = 65536,
  proof_disk_bytes_max | Number = 200000000000,
  proof_jobs | Number = 16,
  follow_up_runs | Array LaterDev = [
    {
      name = "dev-cached-a7841aea", mode = 'resume,
      predecessor = "dev-cold-a7841aea", cache = cache_path,
      requires_prior_report_review = true, requires_empty_output = true,
    },
    {
      name = "dev-adopt-a7841aea", mode = 'adopt,
      predecessor = "dev-cached-a7841aea", cache = cache_path,
      requires_prior_report_review = true, requires_empty_output = true,
    },
  ],
  prefix_checks | Array { report_stage | String, stage_id | String } = [
    { report_stage = "stagex-transition", stage_id = "stagex-transition" },
    { report_stage = "stagex-provider", stage_id = "stagex-provider-publication" },
    { report_stage = "full-source-native-provider", stage_id = "full-source-native-provider" },
    { report_stage = "full-source-rust-provider", stage_id = "full-source-rust-provider" },
    { report_stage = "mantle-stage1", stage_id = "mantle-stage1" },
    { report_stage = "mantle-stage2", stage_id = "mantle-stage2" },
  ],
  promoted = {
    name | String = "promoted-cold-a7841aea",
    source_cohort_must_match | Bool = true,
    requires_separate_plan_review | Bool = true,
    requires_empty_authority | Bool = true,
    permits_dev_cache_access | Bool = false,
    permits_checkpoint_import | Bool = false,
  },
  non_claims | Array String = [
    "This descriptor is not a launch receipt or execution permission.",
    "A shared plan digest does not grant dev-state authority to the promoted run.",
    "Report review does not verify payload bytes, source authority, runtime behavior, or alias state.",
  ],
}
