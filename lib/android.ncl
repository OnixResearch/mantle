# Android APK authoring produces a validated data plan; tool execution is bound by the Rust adapter.
let sources = import "android/sources.ncl" in
let Error = fun error_code error_message => { code = error_code, message = error_message } in
let HasExactly = fun fields value =>
  std.is_record value
  && std.array.length (std.record.fields value) == std.array.length fields
  && std.array.all (fun field => std.record.has_field field value) fields
in
let Meaningful = fun value =>
  std.is_string value && !std.string.is_match "^[ \t\r\n]*$" value
in
let Member = fun value =>
  Meaningful value
  && std.string.is_match "^[A-Za-z0-9._/@+-]+$" value
  && std.array.all
    (fun segment => segment != "" && segment != "." && segment != "..")
    (std.string.split "/" value)
in
# The adapter additionally binds these store-shaped paths to its configured store
# prefix and declared inputs; no ambient filesystem path is admitted by this plan.
let StoreFile = fun value =>
  Meaningful value
  && std.string.is_match
    "^(/[A-Za-z0-9._+-]+)+/[0123456789abcdfghijklmnpqrsvwxyz]{32}-[A-Za-z0-9._+-]+(/[A-Za-z0-9._+-]+)*$"
    value
  && std.array.all
    (fun segment => segment != "." && segment != "..")
    (std.string.split "/" value)
in
let Members = fun value =>
  std.is_array value && std.array.all Member value
in
let Identity = fun component value =>
  let expected = sources.cohort
    |> std.array.filter (fun record => record.component == component)
    |> std.array.first
  in
  HasExactly ["component", "record_blake3", "sha256"] value
  && value.component == expected.component
  && value.record_blake3 == expected.record_blake3
  && value.sha256 == expected.sha256
in
let Toolchains = fun value =>
  HasExactly ["jdk", "build_tools", "platform"] value
  && Identity "jdk" value.jdk
  && Identity "build-tools" value.build_tools
  && Identity "platform-android-jar" value.platform
in
let Schemes = fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.all (fun scheme =>
    scheme == "v1" || scheme == "v2" || scheme == "v3"
  ) value
  && std.array.all (fun scheme =>
    std.array.length (std.array.filter (fun other => other == scheme) value) == 1
  ) value
in
let SigningShape = ["keystore", "store_password_file", "key_password_file", "alias", "schemes"] in
let PlanShape = [
  "module", "application_id", "version_code", "version_name", "manifest",
  "resources", "java_sources", "toolchains", "signing", "reproducibility",
] in
let ValidateApk = fun plan =>
  if !HasExactly PlanShape plan
    || !Meaningful plan.module
    || !Meaningful plan.application_id
    || !std.string.is_match "^[A-Za-z][A-Za-z0-9_]*(\\.[A-Za-z][A-Za-z0-9_]*)+$" plan.application_id
    || !std.is_number plan.version_code
    || !std.number.is_integer plan.version_code
    || plan.version_code < 1 || plan.version_code > 4294967295
    || !Meaningful plan.version_name
    || !Member plan.manifest
    || !Members plan.resources
    || !Members plan.java_sources
  then 'Error (Error "MissingManifestMember" "APK plan requires a valid module, application ID, version, and declared source members")
  else if plan.resources == [] || plan.java_sources == [] then
    'Error (Error "EmptySourceSet" "both resource and Java source member sets are required")
  else if !Toolchains plan.toolchains then
    'Error (Error "UnboundToolchainIdentity" "APK toolchains must match the three pinned Android source records")
  else if !HasExactly ["entry_timestamp_epoch", "locale", "timezone"] plan.reproducibility
    || !std.is_number plan.reproducibility.entry_timestamp_epoch
    || !std.number.is_integer plan.reproducibility.entry_timestamp_epoch
    || plan.reproducibility.entry_timestamp_epoch < 315532800
    || plan.reproducibility.entry_timestamp_epoch > 4354819199
    || plan.reproducibility.locale != "C"
    || plan.reproducibility.timezone != "UTC"
  then 'Error (Error "UnpinnedTimestampPolicy" "APK ZIP entry timestamps require 1980-01-01 through 2107-12-31 UTC, locale C, and timezone UTC")
  else if plan.signing == null then 'Ok
  else if !HasExactly SigningShape plan.signing
    || !StoreFile plan.signing.keystore
    || !StoreFile plan.signing.store_password_file
    || !StoreFile plan.signing.key_password_file
    || !Meaningful plan.signing.alias
  then 'Error (Error "KeystorePathEscape" "signing requires declared absolute store file paths and an alias")
  else if !Schemes plan.signing.schemes then
    'Error (Error "UnknownSignatureScheme" "signing schemes must be unique v1, v2, or v3 values")
  else 'Ok
in
let PlanContract = std.contract.from_validator (fun plan =>
  ValidateApk plan |> match {
    'Ok => 'Ok,
    'Error problem => 'Error { message = "%{problem.code}: %{problem.message}" },
  }
) in
sources & {
  ApkPlan = PlanContract,
  validate_apk = ValidateApk,
  mkApk = fun plan => plan | PlanContract,
}
