# HIP only: AMD LLVM 22.0.0 / ROCm 7.2.3 / gfx900 relocatable device ELF.
# The runner performs the bounded semantic validation before launching tools.
#
# Nickel record contracts may carry at most one field without a definition, so
# the plan shape is an open defaults contract followed by the shape check for
# the required fields.
let Derivation = import "derivation.ncl" in
let { Shape, .. } = import "contracts.ncl" in
let Member = { role : String, member : String } in
let Tool = { role : String, member : String, version : String } in
let PlanFieldsDefaults = {
  schema | String | default = "mantle-gpu-plan-v1",
  backend | String | default = "hip-clang",
  build_platform | String | default = "x86_64-linux",
  generated | Array Member | default = [],
  host_target | String | default = "x86_64-unknown-linux-gnu",
  optimization | Number | default = 2,
  defines | { _ : String } | default = {},
  source_date_epoch | Number | default = 1,
  ..
} in
let PlanFieldsRequired = {
  name : String,
  store_prefix : String,
  roles : { _ : String },
  kernel : Member,
  headers : Array Member,
  clang : Tool,
  host_compiler : Tool,
  inspector : Tool,
  runner : Tool,
  sysroot : String,
  resource_dir : Member,
  toolkit : String,
  toolkit_revision : String,
  device_libraries : Member,
  runtime_libraries : Array Member,
  architecture : String,
  entry_points : Array String,
} in
let PlanFields = std.contract.Sequence [
  PlanFieldsDefaults,
  Shape [
    "schema",
    "backend",
    "build_platform",
    "generated",
    "host_target",
    "optimization",
    "defines",
    "source_date_epoch",
  ] PlanFieldsRequired,
] in
let Plan = std.contract.from_validator (fun value =>
  let p = value | PlanFields in
  if p.schema == "mantle-gpu-plan-v1" && p.backend == "hip-clang"
    && p.build_platform == "x86_64-linux" && p.architecture == "gfx900"
    && p.toolkit_revision == "7.2.3" && p.host_target == "x86_64-unknown-linux-gnu"
    && p.clang.version == "22.0.0" && p.host_compiler.version == "22.0.0"
    && p.inspector.version == "22.0.0" && p.runner.version == "mantle-gpu-v1"
    && std.array.elem p.optimization [0, 1, 2, 3]
  then 'Ok else 'Error { message = "gpu-combination-unsupported: HIP AMD LLVM 22.0.0 / ROCm 7.2.3 / gfx900 only" }
) in
let PlanContract = Plan in
{
  Plan = PlanContract,
  mkGpu = fun value =>
    let plan = (value | PlanFields) | Plan in
    {
      name = plan.name,
      builder = "%{std.record.get plan.runner.role plan.roles}/%{plan.runner.member}",
      args = ["run"],
      system = 'x86_64-linux,
      outputs = ["out"],
      addressing_mode = 'input-addressed,
      inputs = std.record.values plan.roles,
      env = { MANTLE_GPU_PLAN = std.serialize 'Json plan },
    } | Derivation,
}
