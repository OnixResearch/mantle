let Contracts = import "../contracts.ncl" in

({
  schema = "mantle-foreign-provenance-policy-v1",
  max_path_infos = 16,
  max_nodes = 0,
  max_blobs = 16,
  max_blob_bytes = 1024,
  max_total_bytes = 4096,
  max_depth = 8,
  max_findings = 16,
  max_duplicates = 8,
  max_container_entries = 16,
  max_container_expanded_bytes = 1024,
  max_container_depth = 4,
  max_path_bytes = 1024,
  max_shebang_bytes = 256,
  allowed_profile_paths = ["/bin/sh"],
} | Contracts.AuditPolicy | Contracts.ExactAuditPolicy)
