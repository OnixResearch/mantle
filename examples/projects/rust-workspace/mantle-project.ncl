# Dependency-free Rust workspace built through Mantle's offline Cargo lane.

let mantle = import "lib.ncl" in
let seed = import "../../../bootstrap/seed.ncl" in
let rust_toolchain = import "../../../bootstrap/rust.ncl" in
let musl_runtime = import "../../../bootstrap/musl.ncl" in

let source_file = fun relative_path input_name digest => mantle.fetchurl {
  url = "file://./%{relative_path}",
  hash = digest,
  name = input_name,
} in

let workspace_manifest = source_file "Cargo.toml" "rust-project-workspace-manifest" "blake3-0YDVKuHQRNVqJb0VJHlnuT8XeAfRa9yeOM2mwTeTVro=" in
let workspace_lock = source_file "Cargo.lock" "rust-project-workspace-lock" "blake3-sj30sO5/Iz9PrItqRSxGGFfiiJdGCOWWg5j4y5Ped5k=" in
let greeting_manifest = source_file "greeting/Cargo.toml" "rust-project-greeting-manifest" "blake3-daWNtItAwQZUS/bpI4uVmL5linnzUuATOpmbE3/0Lv0=" in
let greeting_source = source_file "greeting/src/lib.rs" "rust-project-greeting-source" "blake3-adpCK1CDVoW49LsL3BWI32jHBOqw+d4IB5ib456Dct0=" in
let app_manifest = source_file "workspace-app/Cargo.toml" "rust-project-app-manifest" "blake3-NNx1/r5q+Bng2bSmKaNYvykhmtE2IxQiMru+7Qy+854=" in
let app_source = source_file "workspace-app/src/main.rs" "rust-project-app-source" "blake3-/WETg9XMVFFRD7y9b8c6Ib4chz/znueDDt0g0DVSzaQ=" in

let workspace_source = {
  name = "rust-workspace-source",
  builder = "/bin/sh",
  inputs = [workspace_manifest, workspace_lock, greeting_manifest, greeting_source, app_manifest, app_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      copy_input() {
        input_name="$1"
        destination="$2"
        source_path=""
        for candidate in "$NIX_STORE"/*-"$input_name"; do
          if [ -f "$candidate" ]; then
            source_path="$candidate"
            break
          fi
        done
        if [ -z "$source_path" ]; then
          echo "fixed Rust source input not mounted: $input_name" >&2
          exit 1
        fi
        $BB cp "$source_path" "$destination"
      }

      $BB mkdir -p "$out/greeting/src" "$out/workspace-app/src"
      copy_input rust-project-workspace-manifest "$out/Cargo.toml"
      copy_input rust-project-workspace-lock "$out/Cargo.lock"
      copy_input rust-project-greeting-manifest "$out/greeting/Cargo.toml"
      copy_input rust-project-greeting-source "$out/greeting/src/lib.rs"
      copy_input rust-project-app-manifest "$out/workspace-app/Cargo.toml"
      copy_input rust-project-app-source "$out/workspace-app/src/main.rs"
    "%
  ],
} | mantle.Derivation in

let rust_workspace_package = mantle.offlineCargoPackage {
  name = "workspace-app",
  src = workspace_source,
  source_name = "rust-workspace-source",
  rust = rust_toolchain,
  rust_name = "rust",
  seed_toolchain = seed.toolchain,
  seed_toolchain_name = seed.name,
  musl = musl_runtime,
  musl_name = "musl",
  binary = "workspace-app",
  profile = 'release,
} in

let workspace_check = {
  name = "rust-workspace-smoke-check",
  builder = "/bin/sh",
  inputs = [rust_workspace_package],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      package_input=""
      for candidate in "$NIX_STORE"/*-workspace-app; do
        if [ -x "$candidate/bin/workspace-app" ]; then
          package_input="$candidate"
          break
        fi
      done
      if [ -z "$package_input" ]; then
        echo 'Rust workspace package was not mounted' >&2
        exit 1
      fi
      default_output=$($package_input/bin/workspace-app)
      named_output=$($package_input/bin/workspace-app Mantle)
      if [ "$default_output" != 'Hello, world!' ] || [ "$named_output" != 'Hello, Mantle!' ]; then
        echo "unexpected Rust workspace output: $default_output / $named_output" >&2
        exit 1
      fi
      if "$package_input/bin/workspace-app" first second; then
        echo 'Rust CLI negative invocation unexpectedly succeeded' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'rust-workspace-smoke-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    source = workspace_source,
    workspace-app = rust_workspace_package,
  },
  checks = { smoke = workspace_check },
  default = { package = "workspace-app" },
} | mantle.Project
