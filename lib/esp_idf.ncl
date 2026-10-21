# ESP-IDF has SDK semantics, not the native Linux CMake adapter's compiler plan.
let Derivation = import "derivation.ncl" in
let { StorePath, .. } = import "contracts.ncl" in
let Relative = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 1024
    && std.string.is_match "^[A-Za-z0-9_+.-]+(/[A-Za-z0-9_+.-]+)*$" value
    && std.array.all (fun p => p != "." && p != "..") (std.string.split "/" value)
  then 'Ok else 'Error { message = "expected nonescaping ESP-IDF member" }
) in
let Digest = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
    && value != "0000000000000000000000000000000000000000000000000000000000000000"
  then 'Ok else 'Error { message = "expected pinned BLAKE3 tree identity" }
) in
let Member = { role : Relative, member : Relative } in
let Plan = {
  name : Relative, host : String, release : String, sdk_commit : String, target : String,
  roles : { _ : { root : StorePath, tree_blake3 : Digest } },
  project : Member, sdk : Member, python : Member, cmake : Member, ninja : Member, compiler : Member,
  python_constraints : Member,
  path : Array Member,
  sdkconfig : Dyn, common_defaults : Array Relative, target_defaults : Array Relative,
  partition_csv : Relative,
  components : Array { name : Relative, source : Member, managed : Bool },
  dependencies_lock : Dyn, source_date_epoch : Number, project_version : Relative,
} in
let validate = fun authored =>
  let p = authored | Plan in
  let members = [p.project, p.sdk, p.python, p.python_constraints, p.cmake, p.ninja, p.compiler] @ p.path
    @ std.array.map (fun c => c.source) p.components in
  if p.host != "x86_64-linux" || p.release != "5.5.1"
    || p.sdk_commit != "fcae32885b0296b32044cb99ecbdc50d98dddb83"
    || !std.array.elem p.target ["esp32s3", "esp32c3"]
    || !std.array.all (fun m => std.record.has_field m.role p.roles) members
    || std.array.length (std.record.fields p.roles) > 128
    || p.source_date_epoch < 0 || std.number.floor p.source_date_epoch != p.source_date_epoch
    || (p.dependencies_lock == null && !std.array.all (fun c => !c.managed) p.components)
  then std.fail_with "esp-idf-plan: unsupported cohort, undeclared role, or missing lock"
  else p
in
let mkFirmware = fun { plan, runner, store_prefix } =>
  let p = validate plan in
  let r = runner | Member in
  if !std.record.has_field r.role p.roles then std.fail_with "esp-idf-plan: runner role undeclared"
  else {
    name = "%{p.name}-%{p.target}-idf-5.5.1",
    builder = "%{p.roles."%{r.role}".root}/%{r.member}",
    args = ["run"], system = 'x86_64-linux, outputs = ["out"], sandbox = 'native,
    addressing_mode = 'input-addressed,
    inputs = std.array.map (fun role => role.root) (std.record.values p.roles),
    env = { MANTLE_ESP_IDF_PLAN = std.serialize 'Json p, MANTLE_ESP_IDF_STORE_PREFIX = store_prefix },
  } | Derivation
in
let PlanContract = Plan in
let MemberContract = Member in
let validateFn = validate in
let mkFirmwareFn = mkFirmware in
{ Plan = PlanContract, Member = MemberContract, validate = validateFn, mkFirmware = mkFirmwareFn }
