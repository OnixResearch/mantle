let Contracts = import "../contracts.ncl" in
{
  schema = "mantle-action-result-runtime-policy-v1",
  hash_algorithm = "BLAKE3",
  sources = { allowed_classes = ["local"], local_enabled = true, http_enabled = false },
  trust = {
    key_source = "configured-trusted-public-keys",
    require_record_signature = true,
    require_pathinfo_signature = true,
    require_producer_identity_match = true,
  },
  limits = {
    max_sources = 1,
    max_candidates = 1,
    max_record_bytes = 1,
    max_index_bytes = 1,
    max_outputs = 1,
    max_signatures = 1,
    http_timeout_ms = 0,
    http_retry_attempts = 1,
  },
  offline = { local_lookup = true, remote_lookup = false },
  publication = {
    local_enabled = true,
    http_enabled = false,
    record_before_index = true,
    atomic_visibility = true,
    immutable_no_clobber = true,
  },
  claims = {
    required_strength = "strong",
    non_claims = ["index-presence-is-not-output-trust"],
  },
  gc = {
    retention_basis = "live-admitted-store-paths",
    candidate_metadata_roots_outputs = false,
    max_metadata_entries = 1,
  },
} | Contracts.Policy
