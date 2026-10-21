# Reusable published source-v1 APK plan. Parameters must be actual admitted
# store members from the generic Cargo unit-plan output and pinned NDK tree.
# Fetching, compilation and signing are deliberately not performed at eval time.
let android = import "android.ncl" in
let identity = fun component =>
  let source = std.array.first (std.array.filter (fun source => source.component == component) android.cohort) in
  { component = source.component, record_blake3 = source.record_blake3, sha256 = source.sha256 }
in
fun { library_store_file, ndk_sysroot, api_level ? 24, signing ? null } =>
let selected_api = api_level in
let selected_sysroot = ndk_sysroot in
let selected_signing = signing in
android.mkApk {
  module = "native-minimal",
  application_id = "org.example.nativeapp",
  version_code = 1,
  version_name = "1.0",
  manifest = "AndroidManifest.xml",
  resources = ["res/values/strings.xml"],
  java_sources = ["src/org/example/nativeapp/MainActivity.java"],
  assets = [{ source_member = "data/welcome.txt", destination = "welcome.txt" }],
  native = {
    abi = "arm64-v8a", api_level = selected_api,
    ndk_revision = "28.2.13676358", ndk_sysroot = selected_sysroot,
    libraries = [{
      store_file = library_store_file,
      filename = "libmantle_android_native_minimal.so",
      required_jni_exports = ["Java_org_example_nativeapp_MainActivity_value"],
    }],
  },
  toolchains = {
    jdk = identity "jdk",
    build_tools = identity "build-tools",
    platform = identity "platform-android-jar",
  },
  signing = selected_signing,
  reproducibility = { entry_timestamp_epoch = 315532802, locale = "C", timezone = "UTC" },
}
