# Exact Zephyr v3.7.0/qemu_cortex_m3 firmware authoring. No west or SDK acquisition.
let Derivation = import "derivation.ncl" in
let { StorePath, .. } = import "contracts.ncl" in
let native = import "cmake.ncl" in
let Member = native.Member in
let Tool = native.Tool in
let Digest = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
  then 'Ok else 'Error { message = "expected independently pinned tree BLAKE3" }
) in
let Input = { root : StorePath, tree_blake3 : Digest } in
let Module = { role : String, revision : String, path : String } in
let Plan = {
  schema : String, name : String, store_prefix : String, build_platform : String,
  roles : { _ : Input }, zephyr : String, sdk : String, project : String,
  sdk_version : String, target : String,
  project_subdir : String, resolved_west : Member,
  west : { schema : String, zephyr_revision : String, modules : { _ : Module } },
  board : String, qualifiers : Array String,
  kconfig : Array Member, overlays : Array Member,
  tools : { cmake : Tool, ninja : Tool, python : Tool, dtc : Tool, host : Array Tool },
  source_date_epoch : Number, jobs : Number,
} in
let validate = fun authored =>
  let p = authored | Plan in
  let tools = [p.tools.cmake, p.tools.ninja, p.tools.python, p.tools.dtc] @ p.tools.host in
  let required = [p.zephyr, p.sdk, p.project, p.resolved_west.role]
    @ std.array.map (fun tool => tool.role) tools
    @ std.array.map (fun member => member.role) (p.kconfig @ p.overlays)
    @ std.array.map (fun module => module.role) (std.record.values p.west.modules) in
  if p.schema != "mantle-zephyr-plan-v1" || p.build_platform != "x86_64-linux"
    || p.sdk_version != "0.16.8" || p.target != "arm-zephyr-eabi"
    || p.board != "qemu_cortex_m3" || std.array.length p.qualifiers != 0
    || p.jobs < 1 || p.jobs > 64 || std.number.floor p.jobs != p.jobs
    || std.array.length (std.record.fields p.roles) > 64
    || std.array.length p.kconfig < 1 || std.array.length p.kconfig > 16
    || std.array.length p.overlays > 16 || std.array.length p.tools.host > 32
    || !std.array.all (fun role => std.record.has_field role p.roles) required
  then std.fail_with "zephyr-plan-invalid: unsupported board/platform or undeclared input"
  else if p.west.schema != "mantle-zephyr-resolved-west-v1"
    || p.west.zephyr_revision != "36940db938a8f4a1e919496793ed439850a221c2"
    || std.array.length (std.record.fields p.west.modules) != 1
    || !std.record.has_field "cmsis" p.west.modules
  then std.fail_with "zephyr-manifest-mismatch: exact v3.7.0 CMSIS closure required"
  else if p.west.modules.cmsis.revision != "4b96cbb174678dcd3ca86e11e1f24bc5f8726da0"
    || p.west.modules.cmsis.path != "modules/hal/cmsis"
    || p.tools.cmake.version != "3.30.5" || p.tools.ninja.version != "1.12.1"
    || p.tools.python.version != "3.11.10" || p.tools.dtc.version != "1.7.0"
    || std.array.length (std.array.filter (fun tool => tool.member == "bin/gperf" && tool.version == "3.1") p.tools.host) != 1
  then std.fail_with "zephyr-cohort-mismatch: exact module and tools required"
  else p
in
let mkFirmware = fun { plan, runner } =>
  let p = validate plan in
  let r = runner | Tool in
  if !std.record.has_field r.role p.roles then std.fail_with "zephyr-missing-role: runner"
  else {
    name = p.name,
    builder = "%{p.roles."%{r.role}".root}/%{r.member}",
    args = ["run"], system = 'x86_64-linux, outputs = ["out"],
    addressing_mode = 'input-addressed,
    inputs = std.array.map (fun input => input.root) (std.record.values p.roles),
    env = { MANTLE_ZEPHYR_PLAN = std.serialize 'Json p },
  } | Derivation
in
let PlanContract = Plan in
let InputContract = Input in
let MemberContract = Member in
let ToolContract = Tool in
let validateFn = validate in
let mkFirmwareFn = mkFirmware in
{ Plan = PlanContract, Input = InputContract, Member = MemberContract, Tool = ToolContract, validate = validateFn, mkFirmware = mkFirmwareFn }
