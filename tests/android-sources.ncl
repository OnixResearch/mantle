# One export exercises admission, deterministic record normalization, fixed fetch,
# and the mandatory toolchain binding on the actual pinned four-component cohort.
let android = import "../lib/android.ncl" in
let [jdk, commandline, build_tools, platform] = android.cohort in
let reordered = {
  platform = 'x86_64-linux,
  unpack_shape = jdk.unpack_shape,
  record_blake3 = jdk.record_blake3,
  sha256 = jdk.sha256,
  url = jdk.url,
  version = jdk.version,
  component = jdk.component,
} in
let missing_url = {
  component = jdk.component,
  version = jdk.version,
  sha256 = jdk.sha256,
  record_blake3 = jdk.record_blake3,
  unpack_shape = jdk.unpack_shape,
  platform = jdk.platform,
} in
let missing_sha256 = {
  component = jdk.component,
  version = jdk.version,
  url = jdk.url,
  record_blake3 = jdk.record_blake3,
  unpack_shape = jdk.unpack_shape,
  platform = jdk.platform,
} in
let accepted = fun value => android.validate_record value |> match { 'Ok => true, 'Error _ => false } in
let rejected = fun code value => android.validate_record value |> match {
  'Ok => false, 'Error error => error.code == code,
} in
let rejected_manifest = fun code value => android.validate_manifest value |> match {
  'Ok => false, 'Error error => error.code == code,
} in
let rejected_binding = fun code value => android.bind_result value |> match {
  'Ok => false, 'Error error => error.code == code,
} in
let example = { name = "android-tool-consumer", builder = "/bin/sh", args = ["-c", "true"] } in
let binding = { identity = jdk, observed_sha256 = jdk.sha256, derivation = example } in
let bound = android.bind binding in
let fetch = android.source jdk in
{
  tests = [
    accepted jdk,
    accepted commandline,
    accepted build_tools,
    accepted platform,
    accepted reordered,
    android.normalized jdk == android.normalized reordered,
    rejected "source-shape" missing_url,
    rejected "source-shape" missing_sha256,
    rejected "source-shape" (jdk & { unreviewed = true }),
    rejected "source-sha256" (jdk & { sha256 | force = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" }),
    rejected "source-sha256-drift" (jdk & { sha256 | force = build_tools.sha256 }),
    rejected "source-version" (jdk & { version | force = "" }),
    rejected "source-drift" (jdk & { version | force = "17.0.18+1" }),
    rejected "source-platform" (jdk & { platform | force = 'aarch64-linux }),
    rejected "source-identity" (jdk & { record_blake3 | force = commandline.record_blake3 }),
    rejected "source-drift" (jdk & { url | force = "https://example.invalid/jdk.tar.gz" }),
    rejected "source-url" (jdk & { url | force = "file:///tmp/unreviewed-jdk.tar.gz" }),
    rejected_manifest "source-duplicate" [jdk, jdk],
    (android.validate_manifest [missing_url, build_tools & { sha256 | force = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" }]
      |> match {
        'Ok => false,
        'Error problem =>
          problem.diagnostics |> std.array.map (fun diagnostic => diagnostic.code)
          == ["source-shape", "source-sha256"],
      }),
    (android.validate_manifest [build_tools & { sha256 | force = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" }, missing_url]
      |> match {
        'Ok => false,
        'Error problem =>
          problem.diagnostics |> std.array.map (fun diagnostic => diagnostic.code)
          == ["source-sha256", "source-shape"],
      }),
    rejected_manifest "source-sha256" [jdk, build_tools & { sha256 | force = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" }],
    rejected_binding "toolchain-identity" { observed_sha256 = jdk.sha256, derivation = example },
    rejected_binding "toolchain-digest" (binding & { observed_sha256 | force = build_tools.sha256 }),
    rejected_binding "source-identity" (binding & { identity.record_blake3 | force = commandline.record_blake3 }),
    rejected_binding "toolchain-sandbox" (binding & { derivation.sandbox | force = 'oci }),
    fetch.builder == "builtin:fetchurl",
    fetch.env.url == jdk.url,
    fetch.fixed_output.hash == jdk.sha256,
    fetch.fixed_output.mode == 'flat,
    bound.inputs == [fetch],
    bound.env.MANTLE_ANDROID_TOOLCHAIN_COMPONENT == jdk.component,
    bound.env.MANTLE_ANDROID_TOOLCHAIN_SHA256 == jdk.sha256,
    bound.env.MANTLE_ANDROID_TOOLCHAIN_RECORD_BLAKE3 == jdk.record_blake3,
    bound.sandbox == 'native,
  ] |> std.test.assert_all,
}
