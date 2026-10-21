# Typed Linux CMake/Ninja authoring. Admission, scheduling and network isolation
# remain the ordinary Mantle derivation/store path; there is no CMake builtin.
let Derivation = import "derivation.ncl" in
let { StorePath, .. } = import "contracts.ncl" in
let Identifier = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 128
    && std.string.is_match "^[A-Za-z0-9_+.-]+$" value && value != "." && value != ".."
  then 'Ok else 'Error { message = "expected bounded CMake identifier" }
) in
let Relative = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 1024
    && std.string.is_match "^[A-Za-z0-9_+.-]+(/[A-Za-z0-9_+.-]+)*$" value
    && std.array.all (fun segment => segment != "." && segment != "..") (std.string.split "/" value)
  then 'Ok else 'Error { message = "cmake-path-escape: expected nonescaping relative CMake member" }
) in
let Member = { role : Identifier, member : Relative } in
let Tool = { role : Identifier, member : Relative, version : String } in
let PositiveJobs = std.contract.from_validator (fun value =>
  if std.is_number value && value >= 1 && value <= 64 && std.number.floor value == value
  then 'Ok else 'Error { message = "CMake jobs must be integer 1..64" }
) in
let Plan = {
  schema : String,
  name : Identifier,
  store_prefix : String,
  source : Identifier,
  source_subdir : String,
  roles : { _ : StorePath },
  tools : { cmake : Tool, ninja : Tool, cc : Tool, cxx : Tool, ar : Tool, ranlib : Tool, linker : Tool },
  compiler : String,
  build_platform : String,
  target : { system : String, processor : String, triple : Identifier },
  sysroot : Identifier,
  runtime_roles : Array Identifier,
  dependencies : Array Identifier,
  host_tools : Array Tool,
  c_includes : Array Member,
  cxx_includes : Array Member,
  toolchain_file : Dyn,
  emulator : Dyn,
  fetch_sources : { _ : Identifier },
  definitions : { _ : String },
  build_targets : Array Identifier,
  outputs : Array Relative,
  build_type : String,
  jobs : PositiveJobs,
  source_date_epoch : Number,
} in
let validate = fun authored =>
  let p = authored | Plan in
  let tools = std.record.values p.tools @ p.host_tools @ (if p.emulator == null then [] else [p.emulator | Tool]) in
  let members = p.c_includes @ p.cxx_includes @ (if p.toolchain_file == null then [] else [p.toolchain_file | Member]) in
  let required = [p.source, p.sysroot] @ p.runtime_roles @ p.dependencies @ std.record.values p.fetch_sources
    @ std.array.map (fun tool => tool.role) tools @ std.array.map (fun member => member.role) members in
  if p.schema != "mantle-cmake-plan-v1" || !std.array.elem p.compiler ["clang", "gcc"]
    || !std.array.elem p.build_platform ["x86_64-linux", "aarch64-linux"]
    || p.target.system != "Linux" || !std.array.elem p.target.processor ["x86_64", "aarch64"]
    || !std.array.all (fun role => std.record.has_field role p.roles) required
    || std.array.length (std.record.fields p.roles) > 64 || std.array.length p.outputs == 0
  then std.fail_with "cmake-plan-invalid: unsupported platform/compiler or undeclared role"
  else p
in
let mkProject = fun { plan, runner } =>
  let p = validate plan in
  let r = runner | Tool in
  if !std.record.has_field r.role p.roles then std.fail_with "cmake-role-undeclared: runner"
  else {
    name = p.name,
    builder = "%{p.roles."%{r.role}"}/%{r.member}",
    args = ["run"],
    system = if p.build_platform == "x86_64-linux" then 'x86_64-linux else 'aarch64-linux,
    outputs = ["out"],
    addressing_mode = 'input-addressed,
    inputs = std.record.values p.roles,
    env = { MANTLE_CMAKE_PLAN = std.serialize 'Json p },
  } | Derivation
in
let nativeInput = fun { root, role, root_env } => {
  inputs = { "%{role}" = root | StorePath },
  env = { "%{root_env}" = "{{role:%{role}}}" },
  path = [],
} in
let PlanContract = Plan in
let ToolContract = Tool in
let MemberContract = Member in
let validateFn = validate in
let mkProjectFn = mkProject in
let nativeInputFn = nativeInput in
{ Plan = PlanContract, Tool = ToolContract, Member = MemberContract, validate = validateFn, mkProject = mkProjectFn, nativeInput = nativeInputFn }
