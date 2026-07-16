# One bounded schema generates C and Rust bindings consumed by two applications.

let mantle = import "lib.ncl" in
let seed = import "../../../bootstrap/seed.ncl" in
let rust_toolchain = import "../../../bootstrap/rust.ncl" in
let musl_runtime = import "../../../bootstrap/musl.ncl" in
let seed_name = seed.name in
let seed_target = seed.target in

let source_file = fun relative_path input_name digest => mantle.fetchurl {
  url = "file://./%{relative_path}",
  hash = digest,
  name = input_name,
} in

let valid_schema = source_file "schema/greeting.schema" "schema-codegen-valid-schema" "blake3-UM6ogRYkqUGuglSe2/R/nkTRVudTSwwuaVyRC8AgEnU=" in
let invalid_schema = source_file "schema/invalid.schema" "schema-codegen-invalid-schema" "blake3-1Z5R+tKtfxiMkH5WDYp3+ZhdFo9tc8Ik3UpQh5At2YQ=" in
let unsafe_schema = source_file "schema/unsafe.schema" "schema-codegen-unsafe-schema" "blake3-/zsR8q0p+btJlYdLkQFapd2MU9vePYUQQ2qcYXGgTmg=" in
let generator_script = source_file "tools/generate-bindings.sh" "schema-codegen-generator" "blake3-YaIOoClAznM4FWz+208lk6fKVdfWw8Yb7WrBQ7O2E1w=" in
let c_main = source_file "c/main.c" "schema-codegen-c-main" "blake3-NLxV7F8BfpPR15vyZiX5Ge3xjgcb9cwztWRWNufhriU=" in
let workspace_manifest = source_file "Cargo.toml" "schema-codegen-workspace-manifest" "blake3-lat0Zpny4hcTLmSLcRYFlwnRWtMNslHiXtyuDtgWJ3I=" in
let workspace_lock = source_file "Cargo.lock" "schema-codegen-workspace-lock" "blake3-OoxmofHsbXil9ryK0MJOI31kO4bcAmdU29WrF0G3oyg=" in
let app_manifest = source_file "app/Cargo.toml" "schema-codegen-app-manifest" "blake3-Zhe7n45Wf2uLFsupCkI4hsDYarYJrdW27vQTM/ZLaQM=" in
let app_main = source_file "app/src/main.rs" "schema-codegen-rust-main" "blake3-oJaAmFP0l7Gx8zyZT9sZPJYq7BGr27bl+nGZb2U5SJw=" in

let generate_bindings = fun output_name schema_input schema_input_name => {
  name = output_name,
  builder = "/bin/sh",
  inputs = [schema_input, generator_script],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TOOLS=/tmp/schema-codegen-tools
      $BB mkdir -p "$TOOLS"
      for command in cat cut grep mkdir printf; do
        $BB ln -sf "$BB" "$TOOLS/$command"
      done
      export PATH="$TOOLS"

      schema_path=""
      generator_path=""
      for candidate in "$NIX_STORE"/*-%{schema_input_name}; do
        if [ -f "$candidate" ]; then
          schema_path="$candidate"
          break
        fi
      done
      for candidate in "$NIX_STORE"/*-schema-codegen-generator; do
        if [ -f "$candidate" ]; then
          generator_path="$candidate"
          break
        fi
      done
      if [ -z "$schema_path" ] || [ -z "$generator_path" ]; then
        echo 'schema or generator input was not mounted' >&2
        exit 1
      fi
      /bin/sh "$generator_path" "$schema_path" "$out"
    "%
  ],
} | mantle.Derivation in

let generated_bindings = generate_bindings "schema-codegen-bindings" valid_schema "schema-codegen-valid-schema" in
let invalid_generation = generate_bindings "schema-codegen-invalid-result" invalid_schema "schema-codegen-invalid-schema" in
let unsafe_generation = generate_bindings "schema-codegen-unsafe-result" unsafe_schema "schema-codegen-unsafe-schema" in

let c_app = {
  name = "schema-c-app",
  builder = "/bin/sh",
  inputs = [generated_bindings, c_main, seed.toolchain],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      bindings_input=""
      main_input=""
      toolchain_input=""
      for candidate in "$NIX_STORE"/*-schema-codegen-bindings; do
        if [ -f "$candidate/c/greeting.c" ]; then bindings_input="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-schema-codegen-c-main; do
        if [ -f "$candidate" ]; then main_input="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-%{seed_name}; do
        if [ -x "$candidate/bin/%{seed_target}-gcc" ]; then toolchain_input="$candidate"; break; fi
      done
      if [ -z "$bindings_input" ] || [ -z "$main_input" ] || [ -z "$toolchain_input" ]; then
        echo 'generated C binding, C main, or compiler was not mounted' >&2
        exit 1
      fi

      GCC="$toolchain_input/bin/%{seed_target}-gcc"
      CFLAGS='-O2 -std=c11 -Wall -Wextra -Werror'
      $BB cp "$main_input" /tmp/main.c
      "$GCC" -I"$bindings_input/c" $CFLAGS -c "$bindings_input/c/greeting.c" -o /tmp/greeting.o
      "$GCC" -I"$bindings_input/c" $CFLAGS -c /tmp/main.c -o /tmp/main.o
      "$GCC" -static $CFLAGS /tmp/main.o /tmp/greeting.o -o /tmp/schema-c-app
      $BB mkdir -p "$out/bin"
      $BB cp /tmp/schema-c-app "$out/bin/schema-c-app"
    "%
  ],
} | mantle.Derivation in

let rust_source = {
  name = "schema-rust-workspace-source",
  builder = "/bin/sh",
  inputs = [generated_bindings, workspace_manifest, workspace_lock, app_manifest, app_main],
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
          if [ -f "$candidate" ]; then source_path="$candidate"; break; fi
        done
        if [ -z "$source_path" ]; then
          echo "fixed Rust source input not mounted: $input_name" >&2
          exit 1
        fi
        $BB cp "$source_path" "$destination"
      }

      bindings_input=""
      for candidate in "$NIX_STORE"/*-schema-codegen-bindings; do
        if [ -f "$candidate/rust/greeting.rs" ]; then bindings_input="$candidate"; break; fi
      done
      if [ -z "$bindings_input" ]; then
        echo 'generated Rust binding was not mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$out/app/src"
      copy_input schema-codegen-workspace-manifest "$out/Cargo.toml"
      copy_input schema-codegen-workspace-lock "$out/Cargo.lock"
      copy_input schema-codegen-app-manifest "$out/app/Cargo.toml"
      copy_input schema-codegen-rust-main "$out/app/src/main.rs"
      $BB cp "$bindings_input/rust/greeting.rs" "$out/app/src/greeting.rs"
    "%
  ],
} | mantle.Derivation in

let rust_app = mantle.offlineCargoPackage {
  name = "schema-rust-app",
  src = rust_source,
  source_name = "schema-rust-workspace-source",
  rust = rust_toolchain,
  rust_name = "rust",
  seed_toolchain = seed.toolchain,
  seed_toolchain_name = seed.name,
  musl = musl_runtime,
  musl_name = "musl",
  binary = "schema-rust-app",
  profile = 'release,
} in

let integration_check = {
  name = "schema-codegen-integration-check",
  builder = "/bin/sh",
  inputs = [c_app, rust_app, generated_bindings],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      c_package=""
      rust_package=""
      binding_package=""
      for candidate in "$NIX_STORE"/*-schema-c-app; do
        if [ -x "$candidate/bin/schema-c-app" ]; then c_package="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-schema-rust-app; do
        if [ -x "$candidate/bin/schema-rust-app" ]; then rust_package="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-schema-codegen-bindings; do
        if [ -f "$candidate/bindings.manifest" ]; then binding_package="$candidate"; break; fi
      done
      if [ -z "$c_package" ] || [ -z "$rust_package" ] || [ -z "$binding_package" ]; then
        echo 'C app, Rust app, or generated bindings were not mounted' >&2
        exit 1
      fi

      c_default=$($c_package/bin/schema-c-app)
      rust_default=$($rust_package/bin/schema-rust-app)
      c_named=$($c_package/bin/schema-c-app Mantle)
      rust_named=$($rust_package/bin/schema-rust-app Mantle)
      if [ "$c_default" != 'Hello, World!' ] || [ "$rust_default" != "$c_default" ]; then
        echo "default outputs differ: C=$c_default Rust=$rust_default" >&2
        exit 1
      fi
      if [ "$c_named" != 'Hello, Mantle!' ] || [ "$rust_named" != "$c_named" ]; then
        echo "named outputs differ: C=$c_named Rust=$rust_named" >&2
        exit 1
      fi
      if "$c_package/bin/schema-c-app" first second; then
        echo 'generated C app accepted extra arguments' >&2
        exit 1
      fi
      if "$rust_package/bin/schema-rust-app" first second; then
        echo 'generated Rust app accepted extra arguments' >&2
        exit 1
      fi
      $BB grep -F 'schema_version=1' "$binding_package/bindings.manifest" >/dev/null
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'schema-codegen-integration-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    bindings = generated_bindings,
    c-app = c_app,
    rust-source = rust_source,
    rust-app = rust_app,
    invalid-schema = invalid_generation,
    unsafe-schema = unsafe_generation,
  },
  checks = { integration = integration_check },
  default = { package = "c-app" },
} | mantle.Project
