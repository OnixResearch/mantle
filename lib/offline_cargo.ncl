# Mantle stdlib: bounded offline Cargo package helper.
#
# This helper lowers a declared Rust package into an ordinary derivation that
# runs Cargo inside Mantle's sandbox with explicit source/toolchain inputs. The
# generated output writes share/mantle/offline-cargo-build.json so JSON build
# reports can surface the bounded evidence class.

let { Name, Input, .. } = import "contracts.ncl" in
let Derivation = import "derivation.ncl" in

let EvidenceSchema = "mantle-offline-cargo-build-evidence-v1" in
let EvidenceClass = "cargo-inside-mantle-sandbox" in
let ProjectBuildStatus = "default-project-build-lane" in
let EvidencePath = "share/mantle/offline-cargo-build.json" in
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
    m%",\n    {\"role\":\"vendored-dependencies\",\"path\":\"$VENDOR\"}"%
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
            $BB mkdir -p /tmp/tools /tmp/home /tmp/build /tmp/cargo-home /tmp/cargo-target
            for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk tr head tail \
                       sort wc expr test basename dirname install find xargs readlink \
                       touch true false tee printf; do
              $BB ln -sf $BB /tmp/tools/$cmd
            done
            export PATH="/tmp/tools"
            export HOME=/tmp/home
            export CARGO_HOME=/tmp/cargo-home
            export CARGO_TARGET_DIR=/tmp/cargo-target

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

            $BB mkdir -p /lib 2>/dev/null || true
            $BB ln -sf "$MUSL/lib/libc.so" /lib/ld-musl-x86_64.so.1 2>/dev/null || true

            export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"
            export LD_LIBRARY_PATH="$TOOLCHAIN/x86_64-linux-musl/lib"
            export LIBRARY_PATH="$TOOLCHAIN/x86_64-linux-musl/lib"
            export RUSTFLAGS="-C link-arg=-L$TOOLCHAIN/x86_64-linux-musl/lib"

            cp -r "$SRC/." /tmp/build/
            chmod -R u+w /tmp/build
            cd /tmp/build

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
  "claim_class": "%{EvidenceClass}",
  "project_build_status": "%{ProjectBuildStatus}",
  "target": "%{package.target}",
  "profile": "%{profile_text}",
  "binary": "%{package.binary}",
  "source_closure": [
    {"role":"package-source","path":"$SRC"},
    {"role":"rust-toolchain","path":"$RUST"},
    {"role":"seed-toolchain","path":"$TOOLCHAIN"},
    {"role":"musl-runtime","path":"$MUSL"}%{evidence_vendor_json package}
  ],
  "toolchain": {
    "cargo": "$RUST/bin/cargo",
    "rustc": "$RUST/bin/rustc",
    "linker": "$TOOLCHAIN/bin/x86_64-linux-musl-gcc"
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
