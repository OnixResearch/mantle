# Prebuilt Android source metadata. Evaluation is pure: no fetch occurs here.
# BLAKE3 is over canonical_record below, not over archive bytes. The manifest
# freshness rail recomputes it; Nickel 1.17 has no BLAKE3 primitive.
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

let set_field = fun name value record =>
  if std.record.has_field name record then std.record.update name value record
  else std.record.insert name value record
in

# The caller must supply a separately reviewed expected identity. This
# generic helper compares the entire canonical preimage as well as its stated
# digest; the reviewed-cohort wrapper in android.ncl selects expected itself.
# The fetcher's fixed-output SHA-256 independently refuses byte drift.
let bind_checked_tool = fun expected admitted consumer =>
  let expected = expected | SourceContract in
  let admitted = admitted | SourceContract in
  if expected.component != admitted.component
    || expected.record_identity_blake3 != admitted.record_identity_blake3
    || expected.sha256 != admitted.sha256
    || render_canonical_record expected != render_canonical_record admitted
  then std.fail_with "Android component %{expected.component} digest drift: expected SHA-256 %{expected.sha256}, BLAKE3 %{expected.record_identity_blake3}; observed SHA-256 %{admitted.sha256}, BLAKE3 %{admitted.record_identity_blake3}"
  else
    let consumer = consumer | Derivation in
    if consumer.sandbox != 'native then
      std.fail_with "Android tool execution requires a native sandbox derivation"
    else
      let inputs = consumer.inputs @ [lower_fetch_source admitted] in
      let env = consumer.env
        |> set_field "ANDROID_TOOLCHAIN_COMPONENT" expected.component
        |> set_field "ANDROID_TOOLCHAIN_SHA256" expected.sha256
        |> set_field "ANDROID_TOOLCHAIN_RECORD_IDENTITY_BLAKE3" expected.record_identity_blake3
      in
      consumer |> std.record.update "inputs" inputs |> std.record.update "env" env
in

{
  Source = SourceContract,
  Manifest = ManifestContract,
  canonical_record = render_canonical_record,
  validate_manifest = check_manifest,
  fetch_source = lower_fetch_source,
  bind_tool = bind_checked_tool,
}
