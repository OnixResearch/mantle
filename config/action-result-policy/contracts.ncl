# Typed policy contracts for shared action-result discovery and publication.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in

let HashAlgorithm = std.contract.from_predicate (fun value =>
  std.is_string value && value == "BLAKE3"
) in

let SourceClass = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "local" || value == "http")
) in

let ClaimStrength = std.contract.from_predicate (fun value =>
  std.is_string value && value == "strong"
) in

{
  SourcePolicy = {
    allowed_classes | Array SourceClass,
    local_enabled | Bool,
    http_enabled | Bool,
  },

  TrustPolicy = {
    key_source | NonEmptyString,
    require_record_signature | Bool,
    require_pathinfo_signature | Bool,
    require_producer_identity_match | Bool,
  },

  LimitPolicy = {
    max_sources | PositiveNumber,
    max_candidates | PositiveNumber,
    max_record_bytes | PositiveNumber,
    max_index_bytes | PositiveNumber,
    max_outputs | PositiveNumber,
    max_signatures | PositiveNumber,
    http_timeout_ms | PositiveNumber,
    http_retry_attempts | PositiveNumber,
  },

  OfflinePolicy = {
    local_lookup | Bool,
    remote_lookup | Bool,
  },

  PublicationPolicy = {
    local_enabled | Bool,
    http_enabled | Bool,
    record_before_index | Bool,
    atomic_visibility | Bool,
    immutable_no_clobber | Bool,
  },

  ClaimPolicy = {
    required_strength | ClaimStrength,
    non_claims | Array NonEmptyString,
  },

  GcPolicy = {
    retention_basis | NonEmptyString,
    candidate_metadata_roots_outputs | Bool,
    max_metadata_entries | PositiveNumber,
  },

  Policy = {
    schema | NonEmptyString,
    hash_algorithm | HashAlgorithm,
    sources | SourcePolicy,
    trust | TrustPolicy,
    limits | LimitPolicy,
    offline | OfflinePolicy,
    publication | PublicationPolicy,
    claims | ClaimPolicy,
    gc | GcPolicy,
  },
}
