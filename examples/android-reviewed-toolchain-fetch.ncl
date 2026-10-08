# Exact original-HTTPS fetch roots for the signed APK proof source bundle.
# Each output is still fixed by the reviewed archive SHA-256 in android.ncl.
let android = import "../lib/android.ncl" in
{
  build_tools = android.reviewed.fetch_toolchain.build_tools,
  jdk = android.reviewed.fetch_toolchain.jdk,
  platform = android.reviewed.fetch_toolchain.platform,
}
