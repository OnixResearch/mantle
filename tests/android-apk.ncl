# Offline APK authoring validates the exported data plan without executing SDK tools.
let android = import "../lib/android.ncl" in
let [jdk_source, commandline, build_tools_source, platform_source] = android.cohort in
let unsigned = {
  module = "demo",
  application_id = "org.example.demo",
  version_code = 1,
  version_name = "1.0",
  manifest = "src/main/AndroidManifest.xml",
  resources = ["src/main/res/values/strings.xml"],
  java_sources = ["src/main/java/org/example/MainActivity.java"],
  toolchains = {
    jdk = { component = jdk_source.component, record_blake3 = jdk_source.record_blake3, sha256 = jdk_source.sha256 },
    build_tools = { component = build_tools_source.component, record_blake3 = build_tools_source.record_blake3, sha256 = build_tools_source.sha256 },
    platform = { component = platform_source.component, record_blake3 = platform_source.record_blake3, sha256 = platform_source.sha256 },
  },
  signing = null,
  reproducibility = {
    entry_timestamp_epoch = 315532800,
    locale = "C",
    timezone = "UTC",
  },
} in
let signing_root = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-android-signing" in
let signature = {
  keystore = "%{signing_root}/signing/release.jks",
  store_password_file = "%{signing_root}/signing/store.pass",
  key_password_file = "%{signing_root}/signing/key.pass",
  alias = "release",
  schemes = ["v1", "v2", "v3"],
} in
let rejected = fun code plan => android.validate_apk plan |> match {
  'Ok => false,
  'Error problem => problem.code == code,
} in
let unsigned_plan = android.mkApk unsigned in
let signed_plan = android.mkApk (unsigned & { signing | force = signature }) in
{
  unsigned_json = std.serialize 'Json unsigned_plan,
  signed_json = std.serialize 'Json signed_plan,
  tests = [
    rejected "MissingManifestMember" (unsigned & { manifest | force = "" }),
    rejected "MissingManifestMember" (unsigned & { manifest | force = "../outside.xml" }),
    rejected "MissingManifestMember" (unsigned & { resources | force = ["/outside.xml"] }),
    rejected "MissingManifestMember" (unsigned & { version_code | force = 4294967296 }),
    rejected "MissingManifestMember" (unsigned & { application_id | force = "../example" }),
    rejected "MissingManifestMember" (unsigned & { undeclared | force = true }),
    rejected "EmptySourceSet" (unsigned & { resources | force = [] }),
    rejected "EmptySourceSet" (unsigned & { java_sources | force = [] }),
    rejected "UnboundToolchainIdentity" (unsigned & { toolchains.jdk.component | force = commandline.component }),
    rejected "UnboundToolchainIdentity" (unsigned & { toolchains.platform.sha256 | force = jdk_source.sha256 }),
    rejected "UnpinnedTimestampPolicy" (unsigned & { reproducibility.entry_timestamp_epoch | force = 315532799 }),
    rejected "UnpinnedTimestampPolicy" (unsigned & { reproducibility.entry_timestamp_epoch | force = 4354819200 }),
    rejected "UnpinnedTimestampPolicy" (unsigned & { reproducibility.locale | force = "en_US.UTF-8" }),
    rejected "UnpinnedTimestampPolicy" (unsigned & { reproducibility.timezone | force = "PST" }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "../release.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { key_password_file | force = "/run/secrets/key.pass" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "signing/release.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "/etc/ssl/release.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "%{signing_root}/../outside.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "~/release.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "file:///signing/release.jks" } }),
    rejected "KeystorePathEscape" (unsigned & { signing | force = signature & { keystore | force = "/nix/store/8cbc11f3a5933818ce0db999ee8ddd9f-android-signing/signing/release.jks" } }),
    rejected "UnknownSignatureScheme" (unsigned & { signing | force = signature & { schemes | force = ["v4"] } }),
    rejected "UnknownSignatureScheme" (unsigned & { signing | force = signature & { schemes | force = ["v2", "v2"] } }),
    rejected "UnknownSignatureScheme" (unsigned & { signing | force = signature & { schemes | force = [] } }),
  ] |> std.test.assert_all,
}
