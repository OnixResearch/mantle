let PositiveInteger = std.contract.from_predicate (fun value => std.is_number value && value % 1 == 0 && value > 0) in

let CacheClosurePolicyContract = {
  schema | [| 'mantle-foreign-cache-closure-policy-v1 |],
  max_paths | PositiveInteger,
  max_references | PositiveInteger,
  max_nar_bytes | PositiveInteger,
  max_depth | PositiveInteger,
} in

let NixStorePrefixes = std.contract.from_predicate (fun value =>
  value == ["/nix/store"]
) in

let NixStorePrefix = std.contract.from_predicate (fun value => value == "/nix/store") in

let PreserveCachePathsMode = std.contract.from_predicate (fun value =>
  value == "preserve-cache-paths-v1"
) in

let PreservedPathTranslationPolicyContract = {
  source_prefixes | NixStorePrefixes,
  target_prefix | NixStorePrefix,
  rewrite_builder | Bool,
  rewrite_args | Bool,
  rewrite_env | Bool,
  rewrite_sources | Bool,
  rewrite_declared_references | Bool,
  allow_embedded_source_payload_rewrite | Bool,
  builtin_mappings | { _ | String },
  output_path_recompute_mode | PreserveCachePathsMode,
  trusted_cache_scopes | Array String,
  allowed_sandbox_capabilities | Array String,
} in

{
  PositiveInteger,
  CacheClosurePolicy = CacheClosurePolicyContract,
  PreservedPathTranslationPolicy = PreservedPathTranslationPolicyContract,
}
