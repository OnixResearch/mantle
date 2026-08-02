let Contracts = import "contracts.ncl" in

{
  schema = "mantle-foreign-provenance-policy-v1",
  max_path_infos = 4096,
  max_nodes = 500001,
  max_blobs = 250000,
  max_blob_bytes = 67108864,
  max_total_bytes = 4294967296,
  max_depth = 128,
  max_findings = 4096,
  max_duplicates = 65536,
  max_container_entries = 65536,
  max_container_expanded_bytes = 1073741824,
  max_container_depth = 8,
  max_path_bytes = 4096,
  max_shebang_bytes = 4096,
  allowed_profile_paths = ["/bin/sh", "/usr/bin/env"],
} | Contracts.AuditPolicy | Contracts.ExactAuditPolicy
