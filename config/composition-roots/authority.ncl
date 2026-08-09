# Reviewed authority for the experimental frontend-neutral composition boundary.
let Limits = {
  max_bindings | Number,
  max_collision_decisions | Number,
  max_entries | Number,
  max_depth | Number,
  max_path_bytes | Number,
  max_file_bytes | Number,
  max_total_file_bytes | Number,
  allow_symlinks | Bool,
} in
let Authority = {
  plan_schema | String,
  policy_schema | String,
  receipt_schema | String,
  merge_policy_version | Number,
  default_limits | Limits,
  unsupported_metadata_classes | Array String,
  forbidden_frontend_fields | Array String,
  non_claim | String,
} in
{
  plan_schema = "mantle-composition-plan-v1",
  policy_schema = "mantle-composition-policy-v1",
  receipt_schema = "mantle-composition-receipt-v1",
  merge_policy_version = 1,
  default_limits = {
    max_bindings = 16,
    max_collision_decisions = 16,
    max_entries = 1024,
    max_depth = 64,
    max_path_bytes = 4096,
    max_file_bytes = 1048576,
    max_total_file_bytes = 16777216,
    allow_symlinks = true,
  },
  unsupported_metadata_classes = [
    "ownership",
    "acl",
    "capability",
    "security-xattr",
    "hard-link",
    "device-node",
  ],
  forbidden_frontend_fields = [
    "packages",
    "providers",
    "inventory",
    "activation",
    "deployment",
    "rollback",
  ],
  non_claim = "composition-does-not-prove-abi-closure-runtime-boot-authorization-deployment-or-release",
} | Authority
