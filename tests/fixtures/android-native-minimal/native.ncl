# The Android lane is the existing admitted generic Cargo unit-plan producer.
# unit_plan contains its exact producer/config/workspace/planner/toolchain/host
# linker/helper/vendor/runtime/native-role inputs. The config's target_linker
# MUST match this declaration; the producer rejects divergent declarations.
let android = import "android.ncl" in
let cargo_units = import "cargo_unit_plan.ncl" in
fun { unit_plan, target } =>
  cargo_units (std.record.insert "target_linker" (android.native.targetLinker target) unit_plan)
