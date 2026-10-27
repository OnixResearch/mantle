# Opt-in Cargo unit-graph producer. The config is a declared store file whose
# tool/source paths must also be present among the explicit inputs below.
# A plan is not a Cargo compilation fallback: rejected plans fail the build.
# `toolchain` is producer/oracle-only. The config's `rustc` must name an
# independently admitted compiler+std root (e.g. packages.cargo-unit-compiler),
# included in runtime_inputs so it is visible to the producer too. Runtime and
# native closures must not depend on the Cargo/oracle roots. Store roots cannot
# be made safe by changing PATH or copying a filtered view of a mixed toolchain.
#
# `native_inputs` (optional) declares native libraries and tools for build
# scripts, keyed by exact Cargo package id (not an ambiguous package name):
#   { "registry+https://github.com/rust-lang/crates.io-index#bzip2-sys@0.1.13+1.0.8" =
#       { inputs = { bzip2 = "/nix/store/…-bzip2", "pkg-config" = "/nix/store/…-pkg-config" },
#         env = { PKG_CONFIG_PATH = "{{role:bzip2}}/lib/pkgconfig" },
#         path = ["{{role:pkg-config}}/bin"] } }
# Values refer to role store paths only through `{{role:<name>}}` markers; a
# marker naming an undeclared role fails evaluation with
# `native-input-role-undeclared`. Role paths become producer inputs, and the
# evaluated declaration reaches the producer as MANTLE_UNIT_NATIVE_INPUTS,
# which must equal the config's `native_inputs`.
# `target_linker` optionally binds a target-only linker, ordered arguments and
# explicit whole store roots. Its executable must lie below one declared root.
# Omitting it preserves the existing host-linker derivation and environment.
let mantle = import "lib.ncl" in
let role_marker = "\\{\\{role:([^}]*)\\}\\}" in
let field_or = fun name fallback record =>
  if std.record.has_field name record then record."%{name}" else fallback
in
# Explicit normalisation instead of a record contract with `default` fields: merging
# defaults into imported (JSON) records trips Mantle's embedded evaluator fixpoint.
let known_native_fields = std.contract.from_predicate (fun declaration =>
  std.array.all (fun field => std.array.elem field ["inputs", "env", "path"]) (std.record.fields declaration)
) in
let normalize_native_input = fun _package raw_declaration =>
  let declaration = raw_declaration | known_native_fields in {
  inputs = std.record.map (fun _role path => path | mantle.StorePath) (field_or "inputs" {} declaration),
  env = std.record.map (fun _key value => value | String) (field_or "env" {} declaration),
  path = std.array.map (fun entry => entry | String) (field_or "path" [] declaration),
} in
let undeclared_roles = fun declaration =>
  ((std.record.values declaration.env) @ declaration.path)
  |> std.array.flat_map (fun value =>
    std.array.map (fun found => std.array.at 0 found.groups) (std.string.find_all role_marker value)
  )
  |> std.array.filter (fun role => !(std.record.has_field role declaration.inputs))
in
let NativeInputs = std.contract.from_validator (fun value =>
  let undeclared =
    std.record.to_array value
    |> std.array.flat_map (fun entry =>
      std.array.map (fun role => "%{entry.field}: %{role}") (undeclared_roles entry.value)
    )
  in
  if undeclared == [] then
    'Ok
  else
    'Error { message = "native-input-role-undeclared: %{std.string.join ", " undeclared}" }
) in
let known_linker_fields = std.contract.from_predicate (fun declaration =>
  std.array.all (fun field => std.array.elem field ["executable", "arguments", "inputs"]) (std.record.fields declaration)
) in
let normalize_target_linker = fun raw =>
  if raw == null then null else
  let declaration = raw | known_linker_fields in
  let normalized = {
    executable = declaration.executable | mantle.StorePath,
    arguments = (field_or "arguments" [] declaration) | Array String,
    inputs = (field_or "inputs" [] declaration) | Array mantle.StorePath,
  } in
  if std.array.any (fun root => std.array.at 0 (std.string.split "%{root}/" normalized.executable) == "") normalized.inputs
  then normalized
  else std.fail_with "unit-plan-host-path: target linker executable has no declared input root"
in
let unique = fun paths =>
  std.array.fold_left (fun seen path => if std.array.elem path seen then seen else seen @ [path]) [] paths
in
fun { goal_name, producer, config, workspace, planner, toolchain, linker, helper, vendor_sources, runtime_inputs ? [], native_inputs ? {}, target_linker ? null } =>
  let native = std.record.map normalize_native_input native_inputs | NativeInputs in
  let target = normalize_target_linker target_linker in
  let runtime = runtime_inputs | Array mantle.StorePath in
  let base_inputs = [producer, config, workspace, planner, toolchain, linker, helper] @ vendor_sources @ runtime in
  let role_inputs =
    std.record.values native
    |> std.array.flat_map (fun declaration => std.record.values declaration.inputs)
    |> unique
    |> std.array.filter (fun path => !(std.array.elem path base_inputs))
  in
  let target_inputs =
    (if target == null then [] else target.inputs)
    |> unique
    |> std.array.filter (fun path => !(std.array.elem path (base_inputs @ role_inputs)))
  in
  let environment = {
    MANTLE_UNIT_CONFIG = config,
    MANTLE_UNIT_NATIVE_INPUTS = std.serialize 'Json native,
    MANTLE_UNIT_RUNTIME_INPUTS = std.serialize 'Json runtime,
  } in
  {
    name = goal_name,
    builder = producer,
    system = 'x86_64-linux,
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    outputs = ["out", "plan", "sources"],
    dynamic_plan_outputs = ["plan"],
    env = if target == null then environment else
      std.record.insert "MANTLE_UNIT_TARGET_LINKER" (std.serialize 'Json target) environment,
    inputs = base_inputs @ role_inputs @ target_inputs,
  } | mantle.Derivation
