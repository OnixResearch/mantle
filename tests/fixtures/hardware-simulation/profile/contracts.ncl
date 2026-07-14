# Example-owned typed contracts for the hardware simulation fixture.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let PositiveInteger = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0
) in

let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in

let ExactFields = fun fields => std.contract.from_predicate (fun value =>
  std.is_record value
  && std.array.all (fun field => std.array.elem field fields) (std.record.fields value)
) in

let ToolRole = std.contract.from_predicate (fun value =>
  std.array.elem value [
    "verilator",
    "cxx-compiler",
    "linker",
    "runtime-support",
    "shell",
  ]
) in

let SourcePackage = {
  id | NonEmptyString,
  locator | NonEmptyString,
  revision | NonEmptyString,
  recursive_blake3 | Blake3Hex,
  object_ref | NonEmptyString,
  sentinel_blake3 | Blake3Hex,
  dependencies | Array NonEmptyString,
  files | Array NonEmptyString,
} in

let HardwareTarget = {
  id | NonEmptyString,
  top_module | NonEmptyString,
  source_packages | Array NonEmptyString,
  source_files | Array NonEmptyString,
  reference_model | NonEmptyString,
} in

let ToolMember = {
  role | ToolRole,
  package | NonEmptyString,
  version | NonEmptyString,
  store_path | NonEmptyString,
  executable | NonEmptyString,
  binary_blake3 | Blake3Hex,
} in

let ToolCohort = {
  schema | NonEmptyString,
  identity_blake3 | Blake3Hex,
  seed_boundary | NonEmptyString,
  system | NonEmptyString,
  members | Array ToolMember,
  closure_paths_blake3 | Blake3Hex,
} in

let GenerationOptions = {
  language | NonEmptyString,
  trace | Bool,
  optimization | NonEmptyString,
  output_prefix | NonEmptyString,
  extra_args | Array NonEmptyString,
} in

let SmokeCase = {
  id | NonEmptyString,
  input | NonEmptyString,
  expected | NonEmptyString,
  max_log_bytes | PositiveInteger,
} in

let ExpectedOutput = {
  name | NonEmptyString,
  kind | NonEmptyString,
} in

let HardwareBounds = {
  max_source_packages | PositiveInteger,
  max_source_files | PositiveInteger,
  max_generated_units | PositiveInteger,
  max_options | PositiveInteger,
  max_smoke_cases | PositiveInteger,
  max_outputs | PositiveInteger,
  max_text_bytes | PositiveInteger,
  max_plan_bytes | PositiveInteger,
  max_log_bytes | PositiveInteger,
  max_actions | PositiveInteger,
} in

let HardwareProfileContract = {
  schema | NonEmptyString,
  profile_id | NonEmptyString,
  selected_target | NonEmptyString,
  source_packages | Array SourcePackage,
  targets | Array HardwareTarget,
  tool_cohort | ToolCohort,
  generation | GenerationOptions,
  compile_flags | Array NonEmptyString,
  link_flags | Array NonEmptyString,
  smoke_cases | Array SmokeCase,
  expected_outputs | Array ExpectedOutput,
  bounds | HardwareBounds,
  support_tier | NonEmptyString,
  non_claims | Array NonEmptyString,
} in

{
  HardwareProfile = HardwareProfileContract,
  ExactHardwareProfile = ExactFields [
    "schema",
    "profile_id",
    "selected_target",
    "source_packages",
    "targets",
    "tool_cohort",
    "generation",
    "compile_flags",
    "link_flags",
    "smoke_cases",
    "expected_outputs",
    "bounds",
    "support_tier",
    "non_claims",
  ],
}
