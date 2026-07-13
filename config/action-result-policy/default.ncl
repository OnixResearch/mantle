let Contracts = import "contracts.ncl" in
let MaxSources = 16 in
let MaxCandidates = 256 in
let MaxRecordBytes = 1048576 in
let MaxIndexBytes = 262144 in
let MaxOutputs = 64 in
let MaxSignatures = 64 in
let HttpTimeoutMs = 5000 in
let HttpRetryAttempts = 4 in
let MaxGcMetadataEntries = 10000 in

{
  schema = "mantle-action-result-runtime-policy-v1",
  hash_algorithm = "BLAKE3",

  sources = {
    allowed_classes = ["local", "http"],
    local_enabled = true,
    http_enabled = true,
  },

  trust = {
    key_source = "configured-trusted-public-keys",
    require_record_signature = true,
    require_pathinfo_signature = true,
    require_producer_identity_match = true,
  },

  limits = {
    max_sources = MaxSources,
    max_candidates = MaxCandidates,
    max_record_bytes = MaxRecordBytes,
    max_index_bytes = MaxIndexBytes,
    max_outputs = MaxOutputs,
    max_signatures = MaxSignatures,
    http_timeout_ms = HttpTimeoutMs,
    http_retry_attempts = HttpRetryAttempts,
  },

  offline = {
    local_lookup = true,
    remote_lookup = false,
  },

  publication = {
    local_enabled = true,
    http_enabled = false,
    record_before_index = true,
    atomic_visibility = true,
    immutable_no_clobber = true,
  },

  claims = {
    required_strength = "strong",
    non_claims = [
      "ca-mapping-presence-is-not-output-trust",
      "executor-correctness",
      "index-presence-is-not-output-trust",
    ],
  },

  gc = {
    retention_basis = "live-admitted-store-paths",
    candidate_metadata_roots_outputs = false,
    max_metadata_entries = MaxGcMetadataEntries,
  },
} | Contracts.Policy
