# Exact original-HTTPS fetch roots for the signed APK proof source bundle.
# Each output is still fixed by the reviewed archive SHA-256 in android.ncl.
let android = import "../lib/android.ncl" in
{
  build_tools = android.fetch_toolchain.build_tools,
  jdk = android.fetch_toolchain.jdk,
  platform = android.fetch_toolchain.platform,
}
