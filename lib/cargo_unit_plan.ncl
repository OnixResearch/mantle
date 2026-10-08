# Opt-in Cargo unit-graph producer. The config is a declared store file whose
# tool/source paths must also be present among the explicit inputs below.
# A plan is not a Cargo compilation fallback: rejected plans fail the build.
let mantle = import "lib.ncl" in
fun { goal_name, producer, config, workspace, planner, toolchain, linker, helper, vendor_sources } =>
  {
    name = goal_name,
    builder = producer,
    system = 'x86_64-linux,
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    outputs = ["out", "plan", "sources"],
    dynamic_plan_outputs = ["plan"],
    env = { MANTLE_UNIT_CONFIG = config },
    inputs = [producer, config, workspace, planner, toolchain, linker, helper] @ vendor_sources,
  } | mantle.Derivation
