# Reviewed Linux x86_64 prebuilt Android source cohort.
# SHA-256 pins downloaded archive bytes. BLAKE3 binds normalized record metadata
# (not archive bytes); see android/sources.ncl and the manifest freshness test.
let sources = import "android/sources.ncl" in
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
let bind_reviewed_tool = fun component admitted consumer =>
  sources.bind_tool (reviewed_source component) admitted consumer
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
  else std.deep_seq plan (std.record.update "signing" signing plan)
in
let fetch_reviewed_toolchain = {
  build_tools = sources.fetch_source reviewed_toolchain.build_tools,
  jdk = sources.fetch_source reviewed_toolchain.jdk,
  platform = sources.fetch_source reviewed_toolchain.platform,
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
