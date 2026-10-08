let Reviewed = (
# Reviewed APK-adapter metadata, owned by the prebuilt-sources manifest in
# android.ncl, is NOT the ADR 0089 source-v1 record identity below. Its
# mantle-android-prebuilt-record-v1 seven-field JSON preimage uses hex SHA-256
# and an archive-kind/root pair; source-v1 uses SRI SHA-256 and six fields.
# Keep these domains under separate `android.reviewed` and `android.cohort`
# APIs; Nickel 1.17 has no BLAKE3 primitive, so manifest freshness is checked
# outside evaluation.
let fetch = import "../fetch.ncl" in
let Derivation = import "../derivation.ncl" in

let MaxComponents = 16 in
let Component = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[a-z][a-z0-9-]{0,47}$" value
  then 'Ok
  else 'Error { message = "expected a lowercase Android component label (1..48 chars, no slash)" }
) in
let Version = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9][A-Za-z0-9.+_-]{0,63}$" value
  then 'Ok
  else 'Error { message = "expected an exact bounded Android version label" }
) in
let ArchiveRoot = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 256
    && std.string.is_match "^[A-Za-z0-9][A-Za-z0-9._+-]*/([A-Za-z0-9][A-Za-z0-9._+-]*/)*$" value
  then 'Ok
  else 'Error { message = "expected a safe relative archive root ending in /, without empty, dot, or parent components" }
) in
let Sha256 = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
    && !(std.string.is_match "^0{64}$" value)
  then 'Ok
  else 'Error { message = "expected a pinned 64-character lowercase SHA-256, not a placeholder" }
) in
let Blake3 = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
    && !(std.string.is_match "^0{64}$" value)
  then 'Ok
  else 'Error { message = "expected a pinned 64-character lowercase BLAKE3 record identity" }
) in
let UpstreamUrl = std.contract.from_validator (fun value =>
  if std.is_string value
    && std.string.is_match "^(https://[A-Za-z0-9.-]+(:[0-9]+)?/[^[:space:][:cntrl:]]+|file:///[^[:space:][:cntrl:]]+)$" value
  then 'Ok
  else 'Error { message = "expected a complete https:// upstream URL (file:/// absolute paths permitted for offline fixtures)" }
) in
let Layout = {
  archive | [| 'zip, 'tar-gz |],
  root | ArchiveRoot,
} in
let SourceContract = {
  component | Component,
  version | Version,
  url | UpstreamUrl,
  sha256 | Sha256,
  record_identity_blake3 | Blake3,
  unpack_shape | Layout,
  platform | [| 'x86_64-linux |],
} in
let ManifestContract = {
  schema | [| 'mantle-android-prebuilt-sources-v1 |],
  sources | Array SourceContract,
} in

# Explicitly ordered JSON string: field ordering in an input record cannot
# change the metadata preimage. The identity field is never part of its hash.
let render_canonical_record = fun source =>
  let source = source | SourceContract in
  std.serialize 'Json [
    "mantle-android-prebuilt-record-v1", source.component, source.version,
    source.url, source.sha256, std.to_string source.unpack_shape.archive,
    source.unpack_shape.root, std.to_string source.platform,
  ]
in

let check_manifest = fun value =>
  let manifest = value | ManifestContract in
  let count = std.array.length manifest.sources in
  if count == 0 || count > MaxComponents then
    std.fail_with "Android source manifest must contain 1..%{std.to_string MaxComponents} components"
  else
    let names = std.array.map (fun source => source.component) manifest.sources in
    let unique = std.array.fold_left (fun seen name =>
      if std.array.elem name seen then seen else seen @ [name]
    ) [] names in
    if std.array.length unique == count then std.deep_seq manifest.sources manifest
    else std.fail_with "duplicate Android source component"
in
let lower_fetch_source = fun source =>
  let source = source | SourceContract in
  std.deep_seq source ((fetch.fetchurl {
    name = "android-%{source.component}-%{source.version}",
    url = source.url,
    hash = source.sha256,
  }) | Derivation)
in

# Validate the adapter's separate metadata domain without granting source-v1
# authority or constructing a tool input. android.ncl binds published cohort
# authority after this check and before any reviewed tool executes.
let validate_reviewed_binding = fun expected admitted =>
  let expected = expected | SourceContract in
  let admitted = admitted | SourceContract in
  if expected.component != admitted.component
    || expected.record_identity_blake3 != admitted.record_identity_blake3
    || expected.sha256 != admitted.sha256
    || render_canonical_record expected != render_canonical_record admitted
  then std.fail_with "Android component %{expected.component} digest drift: expected SHA-256 %{expected.sha256}, BLAKE3 %{expected.record_identity_blake3}; observed SHA-256 %{admitted.sha256}, BLAKE3 %{admitted.record_identity_blake3}"
  else admitted
in

{
  Source = SourceContract,
  Manifest = ManifestContract,
  canonical_record = render_canonical_record,
  validate_manifest = check_manifest,
  fetch_source = lower_fetch_source,
  validate_binding = validate_reviewed_binding,
}
) in
# Pinned Linux/x86_64 Android source cohort. SHA-256 verifies downloaded bytes;
# record_blake3 binds the canonical source declaration, not binary provenance.
let fetch = import "../fetch.ncl" in
let Derivation = import "../derivation.ncl" in
let Required = ["component", "version", "url", "sha256", "record_blake3", "unpack_shape", "platform"] in
let Error = fun error_code error_message => { code = error_code, message = error_message } in
let HasFields = fun fields value =>
  std.is_record value && (fields |> std.array.all (fun name => std.record.has_field name value))
in
let Sha256 = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^sha256-[A-Za-z0-9+/]{43}=$" value
    && value != "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" then 'Ok
  else 'Error { message = "expected a pinned, non-placeholder SHA-256 SRI digest" }
) in
let Blake3 = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value then 'Ok
  else 'Error { message = "expected a 64-character lowercase BLAKE3 record identity" }
) in
let RecordShape = {
  component | String,
  version | String,
  url | String,
  sha256 | Sha256,
  record_blake3 | Blake3,
  unpack_shape | String,
  platform | [| 'x86_64-linux |],
} in
# The serialized array's field order is fixed independently of Nickel record order.
# Hash UTF-8 bytes of this string without a trailing newline using BLAKE3.
let Normalized = fun record =>
  let fields = [record.component, record.version, record.url, record.sha256, record.unpack_shape, "x86_64-linux"] in
  let quoted = fields |> std.array.map (fun value => std.serialize 'Json value) |> std.string.join "," in
  "mantle-android-source-v1\n[%{quoted}]"
in
let Cohort = [
  {
    component = "jdk",
    version = "17.0.17+10",
    url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz",
    sha256 = "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=",
    record_blake3 = "8cbc11f3a5933818ce0db999ee8ddd9f317ea560a375fc92cf87789fff9c1bfb",
    unpack_shape = "jdk-17.0.17+10/",
    platform = 'x86_64-linux,
  },
  {
    component = "sdk-commandline-tools",
    version = "19.0",
    url = "https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip",
    sha256 = "sha256-fsllKAoHMxHDOeVxzV3neLmXUCbPy+efKxzcseFTF+4=",
    record_blake3 = "f9410ec6d005bfd7b38fa88f4128c0fc48fc1cf7af700b964e68668b13ef8913",
    unpack_shape = "cmdline-tools/",
    platform = 'x86_64-linux,
  },
  {
    component = "build-tools",
    version = "35.0.0",
    url = "https://dl.google.com/android/repository/build-tools_r35_linux.zip",
    sha256 = "sha256-vTpJZpEuuLMO0NALDNprZUO5SdX/4AvqVMBMgeFWHYg=",
    record_blake3 = "4756cf34a0b5818a0d37eb093e36d1b8e7d9a2e09854cd120a5bf802f43d80e0",
    unpack_shape = "android-15/",
    platform = 'x86_64-linux,
  },
  {
    component = "platform-android-jar",
    version = "35-r02",
    url = "https://dl.google.com/android/repository/platform-35_r02.zip",
    sha256 = "sha256-CYjKytAbOKGKR7rBSgaV8ka8dsGwbA7rjrDcglqwyOA=",
    record_blake3 = "8ae729174b6f6479ccfd8e22adb61d04598eb578803be9b48befe7bae17aa173",
    unpack_shape = "android-35/",
    platform = 'x86_64-linux,
  },
] in
let lookup = fun name =>
  Cohort |> std.array.filter (fun expected => expected.component == name)
  |> match { [expected] => 'Some expected, _ => 'None }
in
let ValidateRecord = fun candidate =>
  if !HasFields Required candidate then 'Error (Error "source-shape" "Android source missing required fields")
  else if (std.record.fields candidate |> std.array.length) != (Required |> std.array.length) then
    'Error (Error "source-shape" "Android source has undeclared fields")
  else if !std.is_string candidate.component then 'Error (Error "source-component" "component must be a name")
  else if !(std.is_string candidate.version && std.string.length candidate.version > 0) then
    'Error (Error "source-version" "Android source version must be nonempty")
  else if !(std.is_string candidate.url && std.string.is_match "^https://" candidate.url) then
    'Error (Error "source-url" "Android source URL must be a reviewed HTTPS upstream URL")
  else if !(std.is_string candidate.sha256 && std.string.is_match "^sha256-[A-Za-z0-9+/]{43}=$" candidate.sha256)
    || candidate.sha256 == "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" then
    'Error (Error "source-sha256" "Android source SHA-256 is missing, malformed, or a placeholder")
  else if !(std.is_string candidate.record_blake3 && std.string.is_match "^[0-9a-f]{64}$" candidate.record_blake3) then
    'Error (Error "source-blake3" "Android source record identity must be BLAKE3 hex")
  else if candidate.platform != 'x86_64-linux then
    'Error (Error "source-platform" "only x86_64-linux prebuilt tools are admitted")
  else if !(std.is_string candidate.unpack_shape && std.string.length candidate.unpack_shape > 0) then
    'Error (Error "source-unpack" "Android archive root must be declared")
  else lookup candidate.component |> match {
    'None => 'Error (Error "source-component" "unsupported Android source component: %{candidate.component}"),
    'Some expected =>
      if candidate.sha256 != expected.sha256 then
        'Error (Error "source-sha256-drift" "%{candidate.component}: expected SHA-256 %{expected.sha256}, observed %{candidate.sha256}")
      else if candidate.record_blake3 != expected.record_blake3 then
        'Error (Error "source-identity" "%{candidate.component}: expected record BLAKE3 %{expected.record_blake3}, observed %{candidate.record_blake3}")
      else if Normalized candidate != Normalized expected then
        'Error (Error "source-drift" "%{candidate.component}: source declaration differs from pinned URL, version, unpack shape, or platform")
      else 'Ok,
  }
in
let ValidSourceRecord = std.contract.from_validator (fun candidate =>
  ValidateRecord candidate |> match {
    'Ok => 'Ok,
    'Error problem => 'Error { message = "%{problem.code}: %{problem.message}" },
  }
) in
let ValidateManifest = fun records =>
  if !std.is_array records then 'Error (Error "source-manifest" "Android sources must be an array")
  else if records == [] then 'Error (Error "source-manifest" "Android sources must not be empty")
  else
    let rejected = records
      |> std.array.map ValidateRecord
      |> std.array.filter_map (fun result => result |> match { 'Ok => 'None, 'Error problem => 'Some problem })
    in
    rejected |> match {
      [problem, ..] => 'Error (problem & { diagnostics = rejected }),
      [] =>
        if records |> std.array.all (fun current =>
          records |> std.array.filter (fun other => other.component == current.component)
          |> std.array.length == 1)
        then 'Ok
        else let problem = Error "source-duplicate" "Android source components must be unique" in
          'Error (problem & { diagnostics = [problem] }),
    }
in
let ValidManifest = std.contract.from_validator (fun records =>
  ValidateManifest records |> match {
    'Ok => 'Ok,
    'Error problem =>
      let details = if std.record.has_field "diagnostics" problem then
        std.serialize 'Json problem.diagnostics
      else problem.message
      in
      'Error { message = "%{problem.code}: %{details}" },
  }
) in
let FetchSource = fun raw =>
  let record = (raw | ValidSourceRecord) | RecordShape in
  # Flat SHA-256 of the *archive*, not the NAR hash of unpacked contents.
  fetch.fetchurl {
    url = record.url,
    hash = record.sha256,
    name = "android-%{record.component}-%{record.version}",
  }
in
let BindResult = fun params =>
  if !std.is_record params || !(std.record.has_field "identity" params) then
    'Error (Error "toolchain-identity" "Android toolchain identity input is required")
  else
    ValidateRecord params.identity |> match {
      'Error problem => 'Error problem,
      'Ok =>
        if !(std.record.has_field "observed_sha256" params) then
          'Error (Error "toolchain-digest" "observed SHA-256 is required before Android tool execution")
        else if params.identity.sha256 != params.observed_sha256 then
          'Error (Error "toolchain-digest" "%{params.identity.component}: expected SHA-256 %{params.identity.sha256}, observed %{params.observed_sha256}")
        else if !(std.record.has_field "derivation" params) || !HasFields ["name", "builder"] params.derivation then
          'Error (Error "toolchain-derivation" "Android tool execution requires a named sandbox derivation")
        else if std.record.has_field "sandbox" params.derivation && params.derivation.sandbox != 'native then
          'Error (Error "toolchain-sandbox" "Android tools execute only in native sandbox derivations")
        else 'Ok,
    }
in
let ValidBinding = std.contract.from_validator (fun params =>
  BindResult params |> match {
    'Ok => 'Ok,
    'Error problem => 'Error { message = "%{problem.code}: %{problem.message}" },
  }
) in
let Bind = fun raw =>
  let params = raw | ValidBinding in
  let drv = params.derivation in
  let declared_inputs = if std.record.has_field "inputs" drv then drv.inputs else [] in
  let declared_env = if std.record.has_field "env" drv then drv.env else {} in
  let result = drv & {
    inputs | force = declared_inputs @ [FetchSource params.identity],
    env | force = declared_env & {
      MANTLE_ANDROID_TOOLCHAIN_COMPONENT = params.identity.component,
      MANTLE_ANDROID_TOOLCHAIN_SHA256 = params.identity.sha256,
      MANTLE_ANDROID_TOOLCHAIN_RECORD_BLAKE3 = params.identity.record_blake3,
    },
    sandbox | force = 'native,
  } in
  result | Derivation
in
{
  SourceRecord = ValidSourceRecord,
  reviewed = Reviewed,
  Manifest = ValidManifest,
  Binding = ValidBinding,
  cohort = Cohort,
  normalized = Normalized,
  validate_record = ValidateRecord,
  validate_manifest = ValidateManifest,
  bind_result = BindResult,
  source = FetchSource,
  sources = fun records => (records | ValidManifest) |> std.array.map FetchSource,
  bind = Bind,
}
