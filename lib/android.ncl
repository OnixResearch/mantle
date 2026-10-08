let Reviewed = (
# Reviewed prebuilt metadata is a separate authority from the published
# Android source-v1 cohort (ADR 0089). Its seven-field
# mantle-android-prebuilt-record-v1 preimage binds hex SHA-256 and archive
# layout for the APK adapter; it is NOT the six-field source-v1 record identity.
# The reviewed manifest owns this metadata, not the published source cohort.
let published_sources = import "android/sources.ncl" in
let sources = published_sources.reviewed in
let reviewed_manifest = sources.validate_manifest {
  schema = 'mantle-android-prebuilt-sources-v1,
  sources = [
    {
      component = "temurin-jdk",
      version = "17.0.17+10",
      url = "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz",
      sha256 = "992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75",
      record_identity_blake3 = "5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35",
      unpack_shape = { archive = 'tar-gz, root = "jdk-17.0.17+10/" },
      platform = 'x86_64-linux,
    },
    {
      component = "android-cmdline-tools",
      version = "19.0",
      url = "https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip",
      sha256 = "7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee",
      record_identity_blake3 = "15f4b4e15251dc2832ecaedfbee5925237b9a7f5d8ded9d3860d0704cada35b1",
      unpack_shape = { archive = 'zip, root = "cmdline-tools/" },
      platform = 'x86_64-linux,
    },
    {
      component = "android-build-tools",
      version = "35.0.0",
      url = "https://dl.google.com/android/repository/build-tools_r35_linux.zip",
      sha256 = "bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88",
      record_identity_blake3 = "5bc394f89b8d8f36d1d916873b34021bb142db17107460da180e07c4d0b67e7f",
      unpack_shape = { archive = 'zip, root = "android-15/" },
      platform = 'x86_64-linux,
    },
    {
      component = "android-platform",
      version = "35_r02",
      url = "https://dl.google.com/android/repository/platform-35_r02.zip",
      sha256 = "0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0",
      record_identity_blake3 = "d77b395b209a6e6b2becf69a924c19ff2580fb2c2c4113a4b38aa8bb09e126cf",
      unpack_shape = { archive = 'zip, root = "android-35/" },
      platform = 'x86_64-linux,
    },
  ],
} in
let reviewed_source = fun component =>
  let matching = std.array.filter (fun source => source.component == component) reviewed_manifest.sources in
  if std.array.length matching == 1 then std.array.last matching
  else std.fail_with "Android component %{component} is not in the reviewed cohort"
in
# Finite, pinned equivalence of archive bytes across the reviewed hex-SHA
# metadata records and ADR 0089's SRI-SHA source-v1 records. Do not infer a
# canonical identity from the reviewed BLAKE3 digest: the preimages differ.
let cohort_bridge = [
  {
    reviewed = "temurin-jdk", published = "jdk",
    reviewed_version = "17.0.17+10", published_version = "17.0.17+10",
    hex_sha256 = "992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75",
    sri_sha256 = "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=",
    archive = 'tar-gz,
  },
  {
    reviewed = "android-cmdline-tools", published = "sdk-commandline-tools",
    reviewed_version = "19.0", published_version = "19.0",
    hex_sha256 = "7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee",
    sri_sha256 = "sha256-fsllKAoHMxHDOeVxzV3neLmXUCbPy+efKxzcseFTF+4=",
    archive = 'zip,
  },
  {
    reviewed = "android-build-tools", published = "build-tools",
    reviewed_version = "35.0.0", published_version = "35.0.0",
    hex_sha256 = "bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88",
    sri_sha256 = "sha256-vTpJZpEuuLMO0NALDNprZUO5SdX/4AvqVMBMgeFWHYg=",
    archive = 'zip,
  },
  {
    reviewed = "android-platform", published = "platform-android-jar",
    reviewed_version = "35_r02", published_version = "35-r02",
    hex_sha256 = "0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0",
    sri_sha256 = "sha256-CYjKytAbOKGKR7rBSgaV8ka8dsGwbA7rjrDcglqwyOA=",
    archive = 'zip,
  },
] in
let canonical_source_for = fun reviewed =>
  let mappings = std.array.filter (fun row => row.reviewed == reviewed.component) cohort_bridge in
  if std.array.length mappings != 1 then
    std.fail_with "Android reviewed component has no canonical source-v1 bridge: %{reviewed.component}"
  else
    let mapping = std.array.last mappings in
    let candidates = std.array.filter (fun row => row.component == mapping.published) published_sources.cohort in
    if std.array.length candidates != 1 then
      std.fail_with "Android canonical source-v1 cohort is missing %{mapping.published}"
    else
      let canonical = std.array.last candidates in
      published_sources.validate_record canonical |> match {
        'Error error => std.fail_with "Android canonical source-v1 record rejected: %{error.code}",
        'Ok =>
          if reviewed.version != mapping.reviewed_version
            || canonical.version != mapping.published_version
            || reviewed.url != canonical.url
            || reviewed.unpack_shape.root != canonical.unpack_shape
            || reviewed.unpack_shape.archive != mapping.archive
            || reviewed.platform != canonical.platform
            || reviewed.sha256 != mapping.hex_sha256
            || canonical.sha256 != mapping.sri_sha256
          then std.fail_with "Android reviewed source differs from canonical source-v1 cohort: %{reviewed.component}"
          else canonical,
      }
in
let set_field = fun field value record =>
  if std.record.has_field field record then std.record.update field value record
  else std.record.insert field value record
in
let bind_reviewed_tool = fun component admitted consumer =>
  let expected = reviewed_source component in
  let checked = sources.validate_binding expected admitted in
  let canonical = canonical_source_for checked in
  let bound = published_sources.bind {
    identity = canonical,
    observed_sha256 = canonical.sha256,
    derivation = consumer,
  } in
  let env = bound.env
    |> set_field "ANDROID_TOOLCHAIN_COMPONENT" checked.component
    |> set_field "ANDROID_TOOLCHAIN_SHA256" checked.sha256
    |> set_field "ANDROID_TOOLCHAIN_RECORD_IDENTITY_BLAKE3" checked.record_identity_blake3
  in
  bound |> std.record.update "env" env
in

# The authoring value is the exact wire shape consumed by crunch-android-core.
# Executable steps are lowered only after the std adapter has bound independently
# admitted archive paths and checked their bytes; evaluation grants no tool access.
let Member = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 1024
    && std.string.is_match "^[A-Za-z0-9][A-Za-z0-9._+-]*(/[A-Za-z0-9][A-Za-z0-9._+-]*)*$" value
  then 'Ok else 'Error { message = "expected a safe relative APK source member" }
) in
let Module = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 128
    && std.string.is_match "^[A-Za-z0-9_-]+$" value
  then 'Ok else 'Error { message = "expected an APK module identifier" }
) in
let ApplicationId = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value <= 128
    && std.string.is_match "^[A-Za-z0-9_]+(\\.[A-Za-z0-9_]+)+$" value
  then 'Ok else 'Error { message = "expected a dotted application id" }
) in
let Scheme = std.contract.from_validator (fun value =>
  if std.is_string value && (value == "v1" || value == "v2" || value == "v3")
  then 'Ok else 'Error { message = "expected a supported APK signature scheme" }
) in
let Locale = std.contract.from_validator (fun value =>
  if value == "C" then 'Ok else 'Error { message = "expected fixed C locale" }
) in
let Timezone = std.contract.from_validator (fun value =>
  if value == "UTC" then 'Ok else 'Error { message = "expected fixed UTC timezone" }
) in
let Signing = {
  keystore_member | Member,
  key_alias | Module,
  schemes | Array Scheme,
} in
let PlanContract = {
  module | Module,
  application_id | ApplicationId,
  version_code | Number,
  manifest_member | Member,
  resources | Array Member,
  java_sources | Array Member,
  toolchain | {
    build_tools | sources.Source,
    jdk | sources.Source,
    platform | sources.Source,
  },
  signing | Dyn,
  reproducibility | {
    entry_timestamp_epoch | Number,
    locale | Locale,
    timezone | Timezone,
  },
} in
let reviewed_toolchain = {
  build_tools = reviewed_source "android-build-tools",
  jdk = reviewed_source "temurin-jdk",
  platform = reviewed_source "android-platform",
} in
let default_reproducibility = {
  entry_timestamp_epoch = 315532800,
  locale = "C",
  timezone = "UTC",
} in
let set_default = fun field value record =>
  if std.record.has_field field record then record
  else std.record.insert field value record
in
let mk_apk_impl = fun authored =>
  let provided = authored
    |> set_default "toolchain" reviewed_toolchain
    |> set_default "signing" null
    |> set_default "reproducibility" default_reproducibility
  in
  let plan = provided | PlanContract in
  let identity_matches = fun component admitted =>
    let expected = reviewed_source component in
    expected.record_identity_blake3 == admitted.record_identity_blake3
      && sources.canonical_record expected == sources.canonical_record admitted
  in
  let signing = if plan.signing == null then null else plan.signing | Signing in
  let canonical_toolchain = [
    canonical_source_for plan.toolchain.build_tools,
    canonical_source_for plan.toolchain.jdk,
    canonical_source_for plan.toolchain.platform,
  ] in
  if !(identity_matches "android-build-tools" plan.toolchain.build_tools)
    || !(identity_matches "temurin-jdk" plan.toolchain.jdk)
    || !(identity_matches "android-platform" plan.toolchain.platform)
  then std.fail_with "APK toolchain identity is outside the reviewed Android source cohort"
  else if plan.version_code <= 0 || plan.version_code > 4294967295
    || !(std.string.is_match "^[0-9]+$" (std.to_string plan.version_code))
    || plan.reproducibility.entry_timestamp_epoch < 315532800
    || plan.reproducibility.entry_timestamp_epoch > 4354819199
    || !(std.string.is_match "^[0-9]+$" (std.to_string plan.reproducibility.entry_timestamp_epoch))
  then std.fail_with "APK version or entry timestamp is outside the pinned bounds"
  else if std.array.length plan.resources == 0 || std.array.length plan.java_sources == 0
    || std.array.length plan.resources > 256 || std.array.length plan.java_sources > 256
  then std.fail_with "APK resources and Java source sets must each have 1..256 declared members"
  else if signing != null && (std.array.length signing.schemes == 0
    || std.array.length signing.schemes > 3
    || !(std.string.is_match "^keys/" signing.keystore_member))
  then std.fail_with "signed APK requires a keys/ keystore member and 1..3 signature schemes"
  else std.deep_seq canonical_toolchain (std.deep_seq plan (std.record.update "signing" signing plan))
in
let fetch_reviewed_tool = fun reviewed =>
  let canonical = canonical_source_for reviewed in
  std.deep_seq canonical (sources.fetch_source reviewed)
in
let fetch_reviewed_toolchain = {
  build_tools = fetch_reviewed_tool reviewed_toolchain.build_tools,
  jdk = fetch_reviewed_tool reviewed_toolchain.jdk,
  platform = fetch_reviewed_tool reviewed_toolchain.platform,
} in
let lookup_reviewed_source = reviewed_source in
{
  manifest = reviewed_manifest,
  Source = sources.Source,
  Manifest = sources.Manifest,
  canonical_record = sources.canonical_record,
  validate_manifest = sources.validate_manifest,
  fetch_source = sources.fetch_source,
  bind_tool = bind_reviewed_tool,
  reviewed_source = lookup_reviewed_source,
  Plan = PlanContract,
  mkApk = mk_apk_impl,
  toolchain = reviewed_toolchain,
  reproducibility = default_reproducibility,
  fetch_toolchain = fetch_reviewed_toolchain,
}
) in
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
  reviewed | force = Reviewed,
}
