# Typed contracts for identity-bound foreign execution profiles.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let NonNegativeInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value >= 0
) in

let ForeignScope = std.contract.from_predicate (fun value =>
  value == "foreign-bound"
) in

let DeclaredEnvironment = std.contract.from_predicate (fun value =>
  value == "declared-only"
) in

let NetworkMode = std.contract.from_predicate (fun value =>
  value == "deny" || value == "allow-declared"
) in

let FalseOnly = std.contract.from_predicate (fun value =>
  value == false
) in

let EmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value == 0
) in

let ExactFields = fun fields => std.contract.from_predicate (fun value =>
  std.is_record value
  && std.array.all (fun field => std.array.elem field fields) (std.record.fields value)
) in

let ResourceLimits = {
  min_memory_bytes | NonNegativeInteger,
} in

let ExactResourceLimits = ExactFields ["min_memory_bytes"] in

let ExecutionProfileContract = {
  schema | NonEmptyString,
  profile_id | NonEmptyString,
  scope | ForeignScope,
  environment_mode | DeclaredEnvironment,
  fixed_environment | { _ | String },
  protected_variables | Array NonEmptyString,
  provide_bin_sh | Bool,
  work_directory | NonEmptyString,
  network_mode | NetworkMode,
  allow_setid | FalseOnly,
  syscall_exceptions | EmptyArray,
  writable_prefixes | Array NonEmptyString,
  resource_limits | ResourceLimits | ExactResourceLimits,
  unsupported_capabilities | EmptyArray,
} in

{
  ExecutionProfile = ExecutionProfileContract,
  ExactExecutionProfile = ExactFields [
    "schema",
    "profile_id",
    "scope",
    "environment_mode",
    "fixed_environment",
    "protected_variables",
    "provide_bin_sh",
    "work_directory",
    "network_mode",
    "allow_setid",
    "syscall_exceptions",
    "writable_prefixes",
    "resource_limits",
    "unsupported_capabilities",
  ],
}
