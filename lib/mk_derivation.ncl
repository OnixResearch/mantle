# crunch stdlib: mkDerivation.
#
# A function that takes a simplified package description and produces
# a Derivation record with build phases, PATH composition, and metadata.
#
# Usage:
#   let crunch = import "lib.ncl" in
#   let seed = import "seed.ncl" in
#   let stdenv = crunch.mkStdenv seed in
#   stdenv.mkDerivation {
#     pname = "hello",
#     version = "1.0",
#     buildInputs = [seed.gcc],
#     buildPhase = "gcc -o hello hello.c",
#     installPhase = "mkdir -p $out/bin && cp hello $out/bin/",
#   }

let Derivation = import "derivation.ncl" in
let { Input, .. } = import "contracts.ncl" in

# Contract for mkDerivation's input attributes.
let MkDerivationArgs = {
  pname | String
    | doc "Package name (without version).",

  version | String
    | doc "Package version."
    | default = "0",

  builder | String
    | doc "Path to the build shell. Normally set by stdenv."
    | optional,

  src | Input
    | doc "Source input (store path or derivation)."
    | optional,

  buildInputs | Array Input
    | doc "Build-time dependencies. Their /bin dirs are added to PATH."
    | default = [],

  nativeBuildInputs | Array Input
    | doc "Alias for buildInputs (both go to PATH)."
    | default = [],

  unpackPhase | String
    | doc "Shell commands to unpack the source."
    | optional,

  configurePhase | String
    | doc "Shell commands to configure the build."
    | optional,

  buildPhase | String
    | doc "Shell commands to build."
    | optional,

  checkPhase | String
    | doc "Shell commands to run tests."
    | optional,

  installPhase | String
    | doc "Shell commands to install outputs."
    | optional,

  fixupPhase | String
    | doc "Shell commands to run after install."
    | optional,

  doCheck | Bool
    | doc "Whether to run checkPhase."
    | default = false,

  outputs | Array String
    | doc "Output names."
    | default = ["out"],

  env | { _ : String }
    | doc "Extra environment variables."
    | default = {},

  meta | { .. }
    | doc "Package metadata (description, license, homepage)."
    | default = {},

  passthru | { .. }
    | doc "Arbitrary data for downstream consumers."
    | default = {},

  ..  # Allow extra fields without error.
}
in

# Build a colon-separated PATH from an array of store paths.
let make_path = fun inputs =>
  inputs
    |> std.array.map (fun input =>
      if std.is_string input then "%{input}/bin"
      else if std.is_record input && std.record.has_field "name" input then
        # It's a derivation record — we can't know its output path at
        # eval time. Use the store path if available, otherwise skip.
        ""
      else ""
    )
    |> std.array.filter (fun s => std.string.length s > 0)
    |> std.string.join ":"
in

# Default phase implementations. Used when the phase field is absent.
let default_unpackPhase = m%"
  if [ -n "$src" ]; then
    if [ -d "$src" ]; then
      cp -r "$src/." .
    elif [ -f "$src" ]; then
      tar xf "$src"
    fi
    # Enter single subdirectory if one exists (standard tarball layout).
    _dirs=$(find . -maxdepth 1 -mindepth 1 -type d)
    if [ "$(echo "$_dirs" | wc -l)" -eq 1 ]; then
      cd "$_dirs"
    fi
  fi
"% in
let default_configurePhase = m%"
  if [ -x ./configure ]; then
    ./configure --prefix="$out"
  fi
"% in
let default_buildPhase = m%"
  make -j${NIX_BUILD_CORES:-1}
"% in
let default_installPhase = m%"
  make install
"% in

# Resolve a phase: if present and non-empty, use it. If present and
# empty, skip. If absent, use the default.
let resolve_phase = fun attrs name fallback =>
  if std.record.has_field name attrs then
    let val = std.record.get name attrs in
    if std.string.length val > 0 then
      m%"
        echo "-- %{name} --"
        %{val}
      "%
    else ""
  else
    m%"
      echo "-- %{name} --"
      %{fallback}
    "%
in

# Generate the build script from phases.
let make_script = fun attrs =>
  let check_phase =
    if attrs.doCheck && std.record.has_field "checkPhase" attrs then
      m%"
        echo "-- checkPhase --"
        %{attrs.checkPhase}
      "%
    else ""
  in
  let fixup_phase =
    if std.record.has_field "fixupPhase" attrs then
      let val = std.record.get "fixupPhase" attrs in
      if std.string.length val > 0 then
        m%"
          echo "-- fixupPhase --"
          %{val}
        "%
      else ""
    else ""
  in
  m%"
    set -e
    %{resolve_phase attrs "unpackPhase" default_unpackPhase}
    %{resolve_phase attrs "configurePhase" default_configurePhase}
    %{resolve_phase attrs "buildPhase" default_buildPhase}
    %{check_phase}
    %{resolve_phase attrs "installPhase" default_installPhase}
    %{fixup_phase}
  "%
in

# Apply patches over a base record: for each field in `patches`,
# remove it from `base` (if present) then insert the new value.
# This gives Nix-like `//` (update) semantics without merge conflicts.
let apply_patches = fun base patches =>
  std.array.fold_left (fun acc kv =>
    let k = kv.field in
    let v = kv.value in
    acc
      |> (fun r => if std.record.has_field k r then std.record.remove k r else r)
      |> std.record.insert k v
  ) base (std.record.to_array patches)
in

# The mkDerivation implementation, parameterized by a shell path.
let rec mk_derivation_with_shell = fun shell attrs =>
  let resolved = attrs | MkDerivationArgs in
  let the_name = "%{resolved.pname}-%{resolved.version}" in
  let the_builder =
    if std.record.has_field "builder" resolved
      && std.is_string resolved.builder
    then resolved.builder
    else shell
  in
  let all_inputs = resolved.buildInputs @ resolved.nativeBuildInputs in
  let src_inputs =
    if std.record.has_field "src" resolved
    then [resolved.src]
    else []
  in
  let the_path = make_path all_inputs in
  let the_script = make_script resolved in
  let path_env =
    if std.string.length the_path > 0 then { PATH = the_path }
    else {}
  in
  let src_env =
    if std.record.has_field "src" resolved then
      let the_src = resolved.src in
      if std.is_string the_src then { src = the_src }
      else {}
    else {}
  in
  let build_env = { NIX_BUILD_CORES = "4" } in
  let drv = {
    name = the_name,
    builder = the_builder,
    args = ["-c", the_script],
    outputs = resolved.outputs,
    env = build_env & path_env & src_env & resolved.env,
    inputs = src_inputs @ all_inputs,
    pname = resolved.pname,
    version = resolved.version,
    meta = resolved.meta,
    passthru = resolved.passthru,
    overrideAttrs | not_exported = fun f =>
      mk_derivation_with_shell shell (apply_patches resolved (f resolved)),
  } in
  drv | Derivation
in

# Construct a stdenv from a seed toolchain.
#
# The seed must have at least `bash` and `coreutils`. `gcc` is optional
# but expected for C/C++ packages.
let mkStdenv = fun seed =>
  let shell_bin = "%{seed.bash}/bin/bash" in
  let base_inputs = [seed.bash, seed.coreutils] in
  {
    mkDerivation = fun attrs =>
      let user_bi =
        if std.record.has_field "buildInputs" attrs
        then attrs.buildInputs
        else []
      in
      let combined_bi = user_bi @ base_inputs in
      let with_base =
        attrs
          |> (fun r => if std.record.has_field "buildInputs" r then std.record.remove "buildInputs" r else r)
          |> std.record.insert "buildInputs" combined_bi
      in
      mk_derivation_with_shell shell_bin with_base,

    shell = seed.bash,
    coreutils = seed.coreutils,
    cc =
      if std.record.has_field "gcc" seed then seed.gcc
      else if std.record.has_field "cc" seed then seed.cc
      else null,
  }
in

# mkShell: create a dev environment derivation.
#
# The derivation passes contract validation but fails if actually built.
# A future `crunch shell` command will inspect the record to construct
# an interactive environment.
let mkShell = fun params =>
  let resolved_bash =
    if std.record.has_field "bash" params then params.bash
    else "/bin/sh"
  in
  let bi =
    if std.record.has_field "buildInputs" params then params.buildInputs
    else []
  in
  let extra_env =
    if std.record.has_field "env" params then params.env
    else {}
  in
  let the_name =
    if std.record.has_field "name" params then params.name
    else "shell"
  in
  let the_path = make_path bi in
  let path_env =
    if std.string.length the_path > 0 then { PATH = the_path }
    else {}
  in
  {
    name = the_name,
    builder = "%{resolved_bash}/bin/bash" |> (fun b =>
      if std.is_string resolved_bash then b
      else "/bin/sh"
    ),
    args = ["-c", "echo 'error: this derivation is not meant to be built, use crunch shell' >&2; exit 1"],
    inputs = bi,
    env = path_env & extra_env,
  } | Derivation
in

# Dependency injection: pass a package set to a package function.
let callPackage = fun pkgs pkg_fn => pkg_fn pkgs in

{
  include MkDerivationArgs,
  include mk_derivation_with_shell,
  include mkStdenv,
  include mkShell,
  include callPackage,
}
