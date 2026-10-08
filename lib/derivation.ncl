# crunch stdlib: Derivation contract.
#
# Closed record contract describing what the Rust glue layer consumes.
# Only fields the glue layer uses are permitted. Builder-specific
# fields (pname, version, meta, passthru, overrideAttrs) belong in
# the builders package, not here.
# Every field has a doc annotation for `nickel query` discoverability.

let { Name, System, Input, Sandbox, .. } = import "contracts.ncl" in
let { FixedOutput } = import "fixed_output.ncl" in
let Gate = (import "finish_gates.ncl").Gate in
let PositiveNumber = std.contract.from_validator (fun value =>
  if std.is_number value && value > 0 then 'Ok
  else 'Error { message = "expected a positive bounded workspace number" }
) in
let Workspace = {
  schema | String | default = "mantle-stateful-workspace-policy-v1",
  mode | [| 'none, 'immutable-snapshot, 'mutable-session |] | default = 'none,
  fallback | [| 'fail, 'to-none |] | default = 'fail,
  workspace_id | String | optional,
  snapshot_ref | String | optional,
  guest_path | String | default = "/build/.mantle-workspace",
  compatibility | {
    authority_class | String | default = "local-default",
    action_class | String | default = "default",
    toolchain_refs | Array String | default = [],
  } | default = {},
  quota | {
    bytes_max | PositiveNumber | default = 4294967296,
    files_max | PositiveNumber | default = 100000,
    snapshots_max | PositiveNumber | default = 8,
  } | default = {},
  retention | {
    workspace_count_max | PositiveNumber | default = 64,
    idle_generations_max | PositiveNumber | default = 16,
    age_generations_max | PositiveNumber | default = 256,
    quarantine_count_max | PositiveNumber | default = 16,
  } | default = {},
  scrub | {
    sensitive_paths | Array String | default = [".netrc", ".npmrc", ".pypirc"],
    secret_markers | Array String | default = ["authorization", "bearer ", "credential", "password", "private-key", "secret", "token="],
    scan_depth_max | PositiveNumber | default = 128,
    path_bytes_max | PositiveNumber | default = 4096,
    reject_host_paths | Bool | default = true,
  } | default = {},
  snapshot | {
    enabled | Bool | default = false,
    require_clean_scrub | Bool | default = true,
  } | default = {},
  clean_rebuild | {
    enabled | Bool | default = false,
    require_declared_inputs | Bool | default = true,
  } | default = {},
} in
let PlanInputs = std.contract.from_validator (fun value =>
  let checked = value | Array Input in
  let names = checked
    |> std.array.filter (fun input => std.is_record input && std.record.has_field "producer" input)
    |> std.array.map (fun input => input.name) in
  let unique = std.array.fold_left (fun seen name =>
    if std.array.elem name seen then seen else seen @ [name]
  ) [] names in
  if std.array.length names <= 16 && std.array.length unique == std.array.length names
  then 'Ok
  else 'Error { message = "plan-output input names must be unique and at most 16 references are allowed" }
) in


{
  name | Name
    | doc "Derivation name. Appears in the store path.",

  builder | String
    | doc "Absolute path to the build executable.",

  system | System
    | doc "Target platform."
    | default = 'x86_64-linux,

  args | Array String
    | doc "Arguments passed to the builder."
    | default = [],

  outputs | Array String
    | doc "Output names. Each gets its own store path."
    | default = ["out"],

  dynamic_plan_outputs | Array String
    | doc "Output names that may contain native mantle-plan-v1 or mantle-plan-v2 dynamic build plans. Each name must also appear in outputs."
    | default = [],

  finish_gates | Gate
    | doc "Optional explicit adoption of versioned post-install gates. Existing derivations without this field keep their legacy behavior."
    | optional,

  env | { _ : String }
    | doc "Environment variables set during the build. Output paths and system are auto-populated by the glue layer."
    | default = {},

  inputs | PlanInputs
    | doc "Build inputs. Strings are pre-existing store paths; records may be derivations, selected outputs, or a named late-bound plan root `{ name, producer, plan_output, root, unit_output }`. Each plan-root name is unique (at most 16); use `{{mantle-plan-output:<name>}}` only in args/env."
    | default = [],

  fixed_output | FixedOutput
    | doc "Fixed-output derivation parameters. Only set for FODs (fetchers, fixed downloads)."
    | optional,

  addressing_mode | [| 'input-addressed, 'content-addressed |]
    | doc "How output paths are computed. 'input-addressed: from derivation inputs (deterministic before build). 'content-addressed: from build output content (deterministic after build)."
    | default = 'content-addressed,

  sandbox | Sandbox
    | doc "Sandbox preference. Build execution ships only for 'native today; 'oci and 'wasm stay reserved for future backends."
    | default = 'native,

  workspace | Workspace
    | doc "Explicit retained-tool workspace policy. Mutable sessions are practical-only and never publish strong shared action results."
    | optional,
}
