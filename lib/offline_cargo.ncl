# Mantle stdlib: bounded offline Cargo package helper.
#
# This helper lowers a declared Rust package into an ordinary derivation that
# runs Cargo inside Mantle's sandbox with explicit source/toolchain inputs. The
# generated output writes share/mantle/offline-cargo-build.json so JSON build
# reports can surface the bounded evidence class.

let { Name, Input, .. } = import "contracts.ncl" in
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
let input_closure = fun package =>
  if has_vendor package then
    [package.seed_toolchain, package.musl, package.rust, package.src, package.vendor_src]
  else
    [package.seed_toolchain, package.musl, package.rust, package.src]
in
let vendor_setup_script = fun package =>
  if has_vendor package then
    m%"
      VENDOR=$(find_input "%{package.vendor_name}" Cargo.toml)
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
    | doc "Lower a declared Rust package to a sandboxed Cargo --offline --locked derivation."
    = fun raw_package =>
      let package = raw_package | RustOfflineCargoPackage in
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

            TOOLCHAIN=$(find_input "%{package.seed_toolchain_name}" "bin")
            MUSL=$(find_input "%{package.musl_name}" "lib/libc.so")
            RUST=$(find_input "%{package.rust_name}" "bin/cargo")
            SRC=$(find_input "%{package.source_name}" Cargo.toml)
            LOCKFILE="$SRC/Cargo.lock"

            $BB mkdir -p /lib 2>/dev/null || true
            $BB ln -sf "$MUSL/lib/libc.so" /lib/ld-musl-x86_64.so.1 2>/dev/null || true

            export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"
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
            cargo build --locked --offline %{cargo_profile_arg} --bin %{package.binary} --target %{package.target}

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
        },
        inputs = input_closure package,
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
