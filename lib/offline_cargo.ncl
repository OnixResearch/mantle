# Mantle stdlib: bounded offline Cargo package helper.
#
# This helper lowers a declared Rust package into an ordinary derivation that
# runs Cargo inside Mantle's sandbox with explicit source/toolchain inputs. The
# generated output writes share/mantle/offline-cargo-build.json so JSON build
# reports can surface the bounded evidence class.

let { Name, Input, StorePath, .. } = import "contracts.ncl" in
let Derivation = import "derivation.ncl" in

let EvidenceSchema = "mantle-offline-cargo-build-evidence-v2" in
let EvidenceVersion = "2" in
let EvidenceClass = "cargo-inside-mantle-sandbox" in
let ProjectBuildStatus = "default-project-build-lane" in
let EvidencePath = "share/mantle/offline-cargo-build.json" in
let IdentityBlake3Content = "blake3-content" in
let IdentityStorePath = "store-path" in
let NetworkPolicyMode = "offline" in
let NetworkPolicyResult = "undeclared-network-denied" in
let TargetTriple = String in
let Profile = [| 'debug, 'release |] in
let CargoSourceName = Name in
# One invocation-wide native declaration. Unlike unit-plan declarations, this
# environment is inherited by every Cargo child, not scoped to one package.
let field_or = fun name fallback record =>
  if std.record.has_field name record then record."%{name}" else fallback
in
let role_marker = "\\{\\{role:([^}]*)\\}\\}" in
let role_path = "\\{\\{role:[a-z0-9_-]{1,64}\\}\\}[A-Za-z0-9._+/-]*" in
let shell_quote = fun value => "'" ++ std.string.replace "'" "'\\''" value ++ "'" in
let normalize_native = fun raw =>
  let known = std.array.all (fun key => std.array.elem key ["inputs", "env", "path"]) (std.record.fields raw) in
  let roles = std.record.map (fun role path =>
    if !(std.string.is_match "^[a-z0-9_-]{1,64}$" role) then
      std.fail_with "native-input-invalid: invalid role name"
    else if !(std.string.is_match "^(/[A-Za-z0-9._+-]+)+/[a-z0-9]{32}-[A-Za-z0-9+._?=-]+$" path)
      || std.array.elem ".." (std.string.split "/" path) then
      std.fail_with "native-input-host-path: expected a whole store root"
    else path | StorePath
  ) (field_or "inputs" {} raw) in
  let resolve = fun value =>
    let template = value | String in
    let markers = std.string.find_all role_marker template in
    let missing = std.array.any (fun found => !(std.record.has_field (std.array.at 0 found.groups) roles)) markers in
    let outside = std.string.replace_regex role_path "" template in
    if std.string.length template > 16384
      || std.string.is_match "\\x00" template then
      std.fail_with "native-input-invalid: invalid value length or NUL"
    else if missing then std.fail_with "native-input-role-undeclared"
    else if std.string.is_match "/|\\{\\{role:" outside
      || std.array.elem ".." (std.string.split "/" template) then
      std.fail_with "native-input-host-path: path must follow a declared role"
    else std.array.fold_left (fun result role =>
      std.string.replace "{{role:%{role}}}" roles."%{role}" result
    ) template (std.record.fields roles)
  in
  let environment = std.record.map (fun key value =>
    if !(std.string.is_match "^[A-Za-z_][A-Za-z0-9_]*$" key) then
      std.fail_with "native-input-invalid: invalid environment key"
    else if std.array.elem key ["PATH", "OUT_DIR", "TARGET", "HOST", "NUM_JOBS", "OPT_LEVEL", "DEBUG", "PROFILE", "HOME", "TMPDIR", "TMP", "TEMP", "ENV", "BASH_ENV", "LD_PRELOAD", "LD_LIBRARY_PATH", "LIBRARY_PATH"]
      || std.string.is_match "^(CARGO_|DEP_|MANTLE_|RUST|NIX_)" key then
      std.fail_with "native-input-env-reserved: %{key}"
    else resolve value
  ) (field_or "env" {} raw) in
  # Bound outside the record: Nickel records are recursive, so `path = path`
  # inside the result would refer to the field itself.
  let path_entries = std.array.map (fun value =>
    let resolved = resolve value in
    if !(std.string.is_match "^\\{\\{role:" value)
      || std.string.is_match ":" resolved then
      std.fail_with "native-input-host-path: PATH entry must start with a role"
    else resolved
  ) (field_or "path" [] raw) in
  if !known then std.fail_with "native-input-invalid: unknown declaration field"
  else if std.array.length (std.record.fields roles) > 64
    || std.array.length (std.record.fields environment) > 64
    || std.array.length path_entries > 64 then
    std.fail_with "native-input-invalid: declaration exceeds 64 entries"
  else { inputs = std.record.values roles, env = environment, path = path_entries }
in
let RustOfflineCargoPackage = {
  name | Name,
  src | Input,
  source_name | CargoSourceName,
  rust | Input,
  rust_name | CargoSourceName,
  seed_toolchain | Input,
  seed_toolchain_name | CargoSourceName,
  musl | Input,
  musl_name | CargoSourceName,
  binary | Name,
  target | TargetTriple | default = "x86_64-unknown-linux-musl",
  profile | Profile | default = 'release,
  vendor_src | Input | optional,
  vendor_name | CargoSourceName | optional,
  native_inputs | optional,
  # Complete declared observer inventory is identical with collection off/on.
  # The wrapper root and exact compiler/Cargo paths must be declared explicitly.
  wait_tools | optional,
} in

let profile_dir = fun profile =>
  if profile == 'release then "release" else "debug"
in
let profile_arg = fun profile =>
  if profile == 'release then "--release" else ""
in
let cargo_command_profile_json_arg = fun profile =>
  if profile == 'release then
    m%",
    "--release"
    "%
  else
    ""
in
let has_vendor = fun package => std.record.has_field "vendor_src" package in
let has_wait_tools = fun package => std.record.has_field "wait_tools" package in
let wait_tools = fun package =>
  let tools = package.wait_tools in
  {
    wrapper = (tools.wrapper | StorePath) ++ "/bin/mantle-rust-wait-wrapper",
    cargo = tools.cargo | String,
    rustc = tools.rustc | String,
    host = tools.host | String,
    target = package.target,
    runtime_inputs = std.array.map (fun root => root | StorePath) (field_or "runtime_inputs" [] tools),
  }
in
let input_closure = fun package =>
  if has_vendor package then
    [package.seed_toolchain, package.musl, package.rust, package.src, package.vendor_src]
  else
    [package.seed_toolchain, package.musl, package.rust, package.src]
in
let input_path = fun input source_name marker =>
  if std.is_string input then
    "'" ++ (std.string.replace "'" "'\\''" input) ++ "'"
  else
    "$(find_input \"%{source_name}\" \"%{marker}\")"
in
let vendor_setup_script = fun package =>
  if has_vendor package then
    m%"
      VENDOR=%{input_path package.vendor_src package.vendor_name "."}
      $BB mkdir -p /tmp/vendor
      cp -r "$VENDOR/." /tmp/vendor/
      chmod -R u+w /tmp/vendor
      cat >> .cargo/config.toml << CARGOEOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/tmp/vendor"
CARGOEOF
    "%
  else
    ""
in
let evidence_vendor_json = fun package =>
  if has_vendor package then
    m%",
    {"role":"vendored-dependencies","path":"$VENDOR","identity_class":"%{IdentityBlake3Content}"}
    "%
  else
    ""
in
{
  offlineCargoPackage
    | doc "Lower offline Cargo; declare wait_tools {wrapper store root, cargo/rustc executable paths, host triple, optional runtime_inputs store roots} identically for diagnostics off/on."
    = fun raw_package =>
      let package = raw_package | RustOfflineCargoPackage in
      let native = normalize_native (field_or "native_inputs" {} package) in
      let declared_wait_tools = if has_wait_tools package then wait_tools package else {} in
      let native_path = std.string.join ":" native.path in
      let profile_text = profile_dir package.profile in
      let cargo_profile_arg = profile_arg package.profile in
      {
        name = package.name,
        builder = "/bin/sh",
        args = [
          "-c",
          m%"
            set -eu
            BB=/bin/busybox
            TMP_ROOT=/tmp
            TOOLS_DIR="$TMP_ROOT/tools"
            HOME_DIR="$TMP_ROOT/home"
            BUILD_DIR="$TMP_ROOT/build"
            CARGO_HOME_DIR="$TMP_ROOT/cargo-home"
            CARGO_TARGET_DIR_REAL="$TMP_ROOT/cargo-target"
            $BB mkdir -p "$TOOLS_DIR" "$HOME_DIR" "$BUILD_DIR" "$CARGO_HOME_DIR" "$CARGO_TARGET_DIR_REAL"
            for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk tr head tail \
                       sort wc expr test basename dirname install find xargs readlink \
                       touch true false tee printf; do
              $BB ln -sf $BB "$TOOLS_DIR/$cmd"
            done
            export PATH="$TOOLS_DIR"
            export HOME="$HOME_DIR"
            export CARGO_HOME="$CARGO_HOME_DIR"
            export CARGO_TARGET_DIR="$CARGO_TARGET_DIR_REAL"

            find_input() {
              suffix="$1"
              marker="$2"
              found=""
              for candidate in "$NIX_STORE"/*-"$suffix"; do
                if [ -e "$candidate/$marker" ]; then
                  found="$candidate"
                  break
                fi
              done
              if [ -z "$found" ]; then
                echo "ERROR: missing offline Cargo source closure entry $suffix marker $marker" >&2
                exit 1
              fi
              printf '%s\n' "$found"
            }

            TOOLCHAIN=%{input_path package.seed_toolchain package.seed_toolchain_name "bin"}
            MUSL=%{input_path package.musl package.musl_name "lib/libc.so"}
            RUST=%{input_path package.rust package.rust_name "bin/cargo"}
%{if has_wait_tools package then m%"
            test "$RUST/bin/cargo" = %{shell_quote declared_wait_tools.cargo}
            test "$RUST/bin/rustc" = %{shell_quote declared_wait_tools.rustc}
"% else ""}
            export RUSTC="$RUST/bin/rustc"
            SRC=%{input_path package.src package.source_name "Cargo.toml"}
            LOCKFILE="$SRC/Cargo.lock"

            $BB mkdir -p /lib 2>/dev/null || true
            $BB ln -sf "$MUSL/lib/libc.so" /lib/ld-musl-x86_64.so.1 2>/dev/null || true

            export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"%{if native.path == [] then "" else ":" ++ shell_quote native_path}
            export LD_LIBRARY_PATH="$TOOLCHAIN/x86_64-linux-musl/lib"
            export LIBRARY_PATH="$TOOLCHAIN/x86_64-linux-musl/lib"
            export RUSTFLAGS="-C link-arg=-L$TOOLCHAIN/x86_64-linux-musl/lib"

            cp -r "$SRC/." "$BUILD_DIR/"
            chmod -R u+w "$BUILD_DIR"
            cd "$BUILD_DIR"

            mkdir -p .cargo
            cat > .cargo/config.toml << CARGOEOF
[target.%{package.target}]
linker = "x86_64-linux-musl-gcc"

[net]
offline = true
CARGOEOF
%{vendor_setup_script package}
            "$RUST/bin/cargo" build --locked --offline --message-format=json-render-diagnostics %{cargo_profile_arg} --bin %{package.binary} --target %{package.target}

            mkdir -p "$out/bin" "$out/share/mantle"
            cp "$CARGO_TARGET_DIR/%{package.target}/%{profile_text}/%{package.binary}" "$out/bin/%{package.binary}"
            cat > "$out/%{EvidencePath}" << EOF
{
  "schema": "%{EvidenceSchema}",
  "evidence_version": %{EvidenceVersion},
  "claim_class": "%{EvidenceClass}",
  "project_build_status": "%{ProjectBuildStatus}",
  "target": "%{package.target}",
  "profile": "%{profile_text}",
  "binary": "%{package.binary}",
  "native_inputs": $MANTLE_OFFLINE_CARGO_NATIVE_INPUTS,
  "lockfile": {"role":"cargo-lockfile","path":"$LOCKFILE","identity_class":"%{IdentityBlake3Content}"},
  "source_closure": [
    {"role":"package-source","path":"$SRC","identity_class":"%{IdentityBlake3Content}"},
    {"role":"rust-toolchain","path":"$RUST","identity_class":"%{IdentityStorePath}"},
    {"role":"seed-toolchain","path":"$TOOLCHAIN","identity_class":"%{IdentityStorePath}"},
    {"role":"musl-runtime","path":"$MUSL","identity_class":"%{IdentityStorePath}"}%{evidence_vendor_json package}
  ],
  "toolchain": {
    "cargo": "$RUST/bin/cargo",
    "rustc": "$RUST/bin/rustc",
    "linker": "$TOOLCHAIN/bin/x86_64-linux-musl-gcc"
  },
  "cargo_command": {
    "program": "cargo",
    "args": [
    "build",
    "--locked",
    "--offline"%{cargo_command_profile_json_arg package.profile},
    "--message-format=json-render-diagnostics",
    "--bin",
    "%{package.binary}",
    "--target",
    "%{package.target}"
  ]
  },
  "network_policy": {
    "mode": "%{NetworkPolicyMode}",
    "result": "%{NetworkPolicyResult}",
    "allow_undeclared_network": false
  },
  "output": {
    "binary": "%{package.binary}",
    "path": "$out/bin/%{package.binary}"
  },
  "non_claims": [
    "not-cargo-free-execution",
    "not-full-cargo-compatibility",
    "not-compiler-correctness",
    "not-release-reproducibility",
    "not-bootstrap-correctness"
  ]
}
EOF
          "%
        ],
        env = {
          MANTLE_BUILD_EVIDENCE_SCHEMA = EvidenceSchema,
          MANTLE_BUILD_EVIDENCE_CLASS = EvidenceClass,
          MANTLE_PROJECT_BUILD_STATUS = ProjectBuildStatus,
          MANTLE_OFFLINE_CARGO_NATIVE_INPUTS = std.serialize 'Json {
            scope = "cargo-invocation-wide",
            inputs = native.inputs,
            env = native.env,
            path = native.path,
          },
        } & native.env & (if has_wait_tools package then {
          MANTLE_OFFLINE_CARGO_WAIT_TOOLS = std.serialize 'Json declared_wait_tools,
        } else {}),
        inputs = input_closure package @ native.inputs @ (if has_wait_tools package then [package.wait_tools.wrapper] @ declared_wait_tools.runtime_inputs else []),
        addressing_mode = 'input-addressed,
      } | Derivation,

  offlineCargoEvidence = {
    schema = EvidenceSchema,
    evidence_version = EvidenceVersion,
    claim_class = EvidenceClass,
    project_build_status = ProjectBuildStatus,
    evidence_path = EvidencePath,
    non_claims = [
      "not-cargo-free-execution",
      "not-full-cargo-compatibility",
      "not-compiler-correctness",
      "not-release-reproducibility",
      "not-bootstrap-correctness",
    ],
  },
}
