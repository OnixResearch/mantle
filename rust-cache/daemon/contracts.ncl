# Typed policy for the Mantle Rust compiler cache daemon.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "[\n\r]" value
) in
let Identifier = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^[A-Za-z0-9][A-Za-z0-9._:/-]*$" value
) in
let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/.*$" value
  && !std.string.is_match "//|(^|/)\\.\\.?(/|$)" value
  && !std.string.is_match "[[:space:];`$]" value
) in
let CredentialFreeEndpoint = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.is_match "@|[\n\r]" value
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let PositiveInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0
) in
let NonNegativeInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value >= 0
) in
let CacheMode = std.contract.from_predicate (fun value =>
  std.array.elem value ["off", "local-read", "local-read-write", "shared-read", "shared-read-write"]
) in
let FailureMode = std.contract.from_predicate (fun value =>
  std.array.elem value ["fail-open", "fail-closed"]
) in
let SourceKind = std.contract.from_predicate (fun value =>
  std.array.elem value ["directory", "http"]
) in
let Effects = {
  filesystem | std.contract.Equal "declared-roots",
  network | std.contract.Equal "deny",
  clock | std.contract.Equal "allow",
  randomness | std.contract.Equal "allow",
} in
let LimitsContract = {
  elapsed_millis | PositiveInteger,
  stdout_bytes | PositiveInteger,
  stderr_bytes | PositiveInteger,
  artifact_bytes | PositiveInteger,
} in
let ResultSourceContract = {
  source_id | Identifier,
  kind | SourceKind,
  endpoint | CredentialFreeEndpoint,
  priority | NonNegativeInteger,
} in
let PublicationContract = {
  enabled | Bool,
  source_id | Dyn,
  signer_name | Dyn,
  signing_key_path | Dyn,
} in
let PolicyContract = {
  schema | std.contract.Equal "mantle-rustc-wrapper-policy-v1",
  policy_id | Identifier,
  cache_mode | CacheMode,
  failure_mode | FailureMode,
  socket_path | AbsolutePath,
  allowed_peer_uids | Array NonNegativeInteger,
  readable_roots | Array AbsolutePath,
  writable_roots | Array AbsolutePath,
  allowed_environment | Array Identifier,
  effects | Effects,
  limits | LimitsContract,
  max_request_bytes | PositiveInteger,
  max_response_bytes | PositiveInteger,
  max_concurrency | PositiveInteger,
  sandbox_program | AbsolutePath,
  sandbox_program_blake3 | Blake3,
  result_sources | Array ResultSourceContract,
  shared_trust_policy | Dyn,
  publication | PublicationContract,
  local_reads_enabled | Bool,
  local_writes_enabled | Bool,
  shared_reads_enabled | Bool,
  shared_writes_enabled | Bool,
  redact_environment_values | std.contract.Equal true,
} in
{
  Policy = PolicyContract,
  Limits = LimitsContract,
  ResultSource = ResultSourceContract,
  Publication = PublicationContract,
}
