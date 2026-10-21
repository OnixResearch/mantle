# Explicit driver policy admitted in the executable inventory, never PATH injection.
let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^/.*$" value
  && !std.string.is_match "//|(^|/)\\.\\.?(/|$)|[\n\r]" value
) in
let Identifier = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[A-Za-z0-9-]{1,128}$" value
) in
let BoundedMillis = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0 && value <= 3600000
) in
let Root = { alias | Identifier, path | AbsolutePath, path_independent | Bool } in
let Policy = {
  schema | std.contract.Equal "mantle-cc-driver-policy-v1",
  compiler | AbsolutePath,
  roots | Array Root,
  platform | String,
  environment | { _ | String },
  probe_script | std.contract.AnyOf [std.contract.Equal null, AbsolutePath],
  cache_enabled | Bool,
  slots_enabled | Bool,
  output_path_independent | Bool,
  elapsed_ms_max | BoundedMillis,
  slot_wait_ms_max | BoundedMillis,
  receipt_dir | AbsolutePath,
} in
{
  Policy = Policy,
  Root = Root,
  contract = {
    schema = "mantle-cc-cache-contract-v1",
    driver_policy_schema = "mantle-cc-driver-policy-v1",
    daemon_request_schema = "mantle-cc-cache-request-v1",
    receipt_schema = "mantle-cc-driver-receipt-v1",
    key = "blake3-content-tool-options-platform-environment-complete-roots-learned-dependencies",
    unknown_invocation = "forward-original-argv",
    endpoint = "execution-only-unix-socket",
    authority = "machine-local-performance-only",
    proof_evidence = false,
    defaults_enabled = false,
    bounds = { roots = 64, records = 256, object_bytes = 8388608,
      diagnostic_bytes = 262144, dependency_count = 4096 },
    non_claims = ["compiler-correctness", "rebuild-evidence", "remote-trust", "derivation-input"],
  },
}
