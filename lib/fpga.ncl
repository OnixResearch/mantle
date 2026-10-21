# Typed synthesis/placement inputs. Rust performs bounded admission and semantic
# validation again; this contract grants no build or device-programming authority.
#
# Nickel record contracts may carry at most one field without a definition, so
# shapes with several required fields are expressed as a record type, or as an
# open defaults contract followed by the shape check for the required fields.
let { Shape, .. } = import "contracts.ncl" in
let NonEmpty = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value > 0 then 'Ok
  else 'Error { message = "expected nonempty string" }) in
let Blake3 = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value then 'Ok
  else 'Error { message = "expected lowercase BLAKE3" }) in
let Absolute = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^/" value then 'Ok
  else 'Error { message = "expected absolute path" }) in
let UInt = std.contract.from_validator (fun value =>
  if std.is_number value && value >= 0 && value <= 4294967295 && std.number.floor value == value then 'Ok
  else 'Error { message = "expected unsigned 32-bit integer" }) in
let FileInput = { name : NonEmpty, path : Absolute, blake3 : Blake3 } in
let Tool = { executable : FileInput, version : NonEmpty, closure : Array Absolute } in
let Clock = { port : NonEmpty, report_net : NonEmpty, frequency_khz : UInt } in
let Pass = std.contract.from_validator (fun value =>
  if std.array.elem value ["check", "opt", "clean"] then 'Ok
  else 'Error { message = "unsupported synthesis pass" }) in
let ProfileDefaults = {
  schema | String = "mantle-fpga-profile-v1",
  part | String = "ice40-hx1k",
  ..
} in
let ProfileRequired = {
  rtl : Array FileInput,
  includes : Array FileInput,
  include_dirs : Array String,
  top : NonEmpty,
  defines : { _ : String },
  parameters : { _ : UInt },
  synthesis_script : FileInput,
  synthesis_passes : Array Pass,
  flatten : Bool,
  pins : { _ : String },
  clocks : Array Clock,
  constraints : FileInput,
  seed : UInt,
  threads : UInt,
  yosys : Tool,
  nextpnr : Tool,
  packer : Tool,
  shell : Tool,
  bwrap : Tool,
  database : { part : String, package : String, nextpnr_binary_blake3 : Blake3 },
  utilization_limits : { _ : UInt },
  timeout_ms : UInt,
  max_log_bytes : UInt,
  max_output_bytes : UInt,
} in
let ProfileContract = std.contract.Sequence [
  ProfileDefaults,
  Shape [ "schema", "part" ] ProfileRequired,
] in
let FileInputContract = FileInput in
let ToolContract = Tool in
let ClockContract = Clock in
{ FileInput = FileInputContract, Tool = ToolContract, Clock = ClockContract, Profile = ProfileContract }
