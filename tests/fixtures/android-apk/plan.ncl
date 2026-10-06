# A tiny real app input set for the offline APK adapter. This plan does not
# execute prebuilt tools or claim an APK until Mantle builds the rendered graph.
let android = import "../../../lib/android.ncl" in
let [jdk_source, _, build_tools_source, platform_source] = android.cohort in
android.mkApk {
  module = "mantle-offline",
  application_id = "org.example.mantle",
  version_code = 1,
  version_name = "1.0",
  manifest = "AndroidManifest.xml",
  resources = ["res/values/strings.xml"],
  java_sources = ["java/org/example/mantle/MainActivity.java"],
  toolchains = {
    jdk = {
      component = jdk_source.component,
      record_blake3 = jdk_source.record_blake3,
      sha256 = jdk_source.sha256,
    },
    build_tools = {
      component = build_tools_source.component,
      record_blake3 = build_tools_source.record_blake3,
      sha256 = build_tools_source.sha256,
    },
    platform = {
      component = platform_source.component,
      record_blake3 = platform_source.record_blake3,
      sha256 = platform_source.sha256,
    },
  },
  signing = null,
  reproducibility = { entry_timestamp_epoch = 315532800, locale = "C", timezone = "UTC" },
}
