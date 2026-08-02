# Typed contract for bounded foreign provenance audit policy.

let PositiveInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0
) in

let AbsoluteLexicalPath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/" value
  && !std.string.is_match "(^|/)\\.\\.(/|$)" value
) in

let ExactFields = fun fields => std.contract.from_predicate (fun value =>
  std.is_record value
  && std.array.all (fun field => std.array.elem field fields) (std.record.fields value)
) in

let AuditPolicyContract = {
  schema | std.contract.from_predicate (fun value => value == "mantle-foreign-provenance-policy-v1"),
  max_path_infos | PositiveInteger,
  max_nodes | PositiveInteger,
  max_blobs | PositiveInteger,
  max_blob_bytes | PositiveInteger,
  max_total_bytes | PositiveInteger,
  max_depth | PositiveInteger,
  max_findings | PositiveInteger,
  max_duplicates | PositiveInteger,
  max_container_entries | PositiveInteger,
  max_container_expanded_bytes | PositiveInteger,
  max_container_depth | PositiveInteger,
  max_path_bytes | PositiveInteger,
  max_shebang_bytes | PositiveInteger,
  allowed_profile_paths | Array AbsoluteLexicalPath,
} in

{
  AuditPolicy = AuditPolicyContract,
  ExactAuditPolicy = ExactFields [
    "schema",
    "max_path_infos",
    "max_nodes",
    "max_blobs",
    "max_blob_bytes",
    "max_total_bytes",
    "max_depth",
    "max_findings",
    "max_duplicates",
    "max_container_entries",
    "max_container_expanded_bytes",
    "max_container_depth",
    "max_path_bytes",
    "max_shebang_bytes",
    "allowed_profile_paths",
  ],
}
