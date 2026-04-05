# crunch stdlib: fetch helpers.
#
# Produce fixed-output derivation records for downloading files,
# tarballs, and git repositories. The Rust orchestrator recognises
# `builder = "builtin:fetchurl"` and executes the fetch directly
# (no sandbox).

let contracts = import "contracts.ncl" in
let Derivation = import "derivation.ncl" in

let Url
  | doc "Validates a URL string (http://, https://, or file://)."
  = std.contract.from_validator (fun value =>
    if std.is_string value
      && std.string.length value > 0
      && std.string.is_match "^(https?|file)://" value
    then 'Ok
    else 'Error { message = "expected a URL (http://, https://, or file://), got: %{std.serialize 'Json value}" }
  )
in

let Hash
  | doc "Validates an SRI hash (algo-base64) or a hex hash (64+ hex chars for sha256)."
  = std.contract.from_validator (fun value =>
    if std.is_string value
      && std.string.length value > 0
      && (std.string.is_match "^(sha256|sha512|sha1|md5|blake3)-[A-Za-z0-9+/]+=*$" value
          || std.string.is_match "^[0-9a-f]{32,}$" value)
    then 'Ok
    else 'Error { message = "expected an SRI hash (sha256-...) or hex hash, got: %{std.serialize 'Json value}" }
  )
in

# Extract the filename from a URL (everything after the last /).
let url_basename = fun url =>
  let parts = std.string.split "/" url in
  std.array.last parts
in

{
  fetchurl
    | doc "Download a file by URL. Hash is over raw file bytes (flat mode)."
    = fun params =>
      let the_url = params.url in
      let the_hash = params.hash in
      let the_name =
        if std.record.has_field "name" params
        then params.name
        else url_basename the_url
      in
      let the_fixed_output = { hash = the_hash, algo = 'sha256, mode = 'flat } in
      let the_env = { url = the_url } in
      {
        name = the_name,
        builder = "builtin:fetchurl",
        system = 'x86_64-linux,
        args = [],
        outputs = ["out"],
        env = the_env,
        inputs = [],
        fixed_output = the_fixed_output,
        addressing_mode = 'input-addressed,
        sandbox = 'native,
      },

  fetchTarball
    | doc "Download a tarball, decompress, extract, strip top-level dir. Hash is NAR of unpacked tree."
    = fun params =>
      let the_url = params.url in
      let the_hash = params.hash in
      let the_name =
        if std.record.has_field "name" params
        then params.name
        else
          url_basename the_url
            |> std.string.replace ".tar.gz" ""
            |> std.string.replace ".tar.xz" ""
            |> std.string.replace ".tar.bz2" ""
            |> std.string.replace ".tar.zst" ""
            |> std.string.replace ".tgz" ""
            |> std.string.replace ".txz" ""
      in
      let the_fixed_output = { hash = the_hash, algo = 'sha256, mode = 'recursive } in
      let the_env = { url = the_url, unpack = "1" } in
      {
        name = the_name,
        builder = "builtin:fetchurl",
        system = 'x86_64-linux,
        args = [],
        outputs = ["out"],
        env = the_env,
        inputs = [],
        fixed_output = the_fixed_output,
        addressing_mode = 'input-addressed,
        sandbox = 'native,
      },

  fetchGit
    | doc "Clone a git repository at a specific revision. Hash is NAR of the checkout (no .git/)."
    = fun params =>
      let the_url = params.url in
      let the_rev = params.rev in
      let the_hash = params.hash in
      let the_name =
        if std.record.has_field "name" params
        then params.name
        else
          url_basename the_url
            |> std.string.replace ".git" ""
      in
      let the_fixed_output = { hash = the_hash, algo = 'sha256, mode = 'recursive } in
      let the_env = { url = the_url, type = "git", rev = the_rev } in
      {
        name = the_name,
        builder = "builtin:fetchurl",
        system = 'x86_64-linux,
        args = [],
        outputs = ["out"],
        env = the_env,
        inputs = [],
        fixed_output = the_fixed_output,
        addressing_mode = 'input-addressed,
        sandbox = 'native,
      },

  # Re-export contracts for external use.
  Url = Url,
  Hash = Hash,
}
