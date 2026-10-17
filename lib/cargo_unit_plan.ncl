# Opt-in offline Cargo unit-graph producer. Native inputs are explicit,
# package-id-keyed data, never discovered through ambient host paths.
let mantle = import "lib.ncl" in
let NativeInput = {
  inputs | { _ : mantle.StorePath },
  env | { _ : String } | default = {},
  path | Array String | default = [],
} in
let NativeInputs = { _ : NativeInput } in
fun { goal_name, producer, config, workspace, planner, toolchain, linker, helper, vendor_sources, runtime_inputs, native_inputs } =>
  let declared = native_inputs | NativeInputs in
  let native_paths = std.array.fold_left
    (fun paths entry => paths @ std.record.values entry.inputs)
    [] (std.record.values declared) in
  {
    name = goal_name,
    builder = producer,
    system = 'x86_64-linux,
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    outputs = ["out", "plan", "sources"],
    dynamic_plan_outputs = ["plan"],
    env = {
      MANTLE_UNIT_CONFIG = config,
      MANTLE_UNIT_NATIVE_INPUTS = std.serialize 'Json declared,
      MANTLE_UNIT_RUNTIME_INPUTS = std.serialize 'Json runtime_inputs,
    },
    inputs = [producer, config, workspace, planner, toolchain, linker, helper] @ vendor_sources @ (runtime_inputs | Array mantle.StorePath) @ native_paths,
  } | mantle.Derivation
