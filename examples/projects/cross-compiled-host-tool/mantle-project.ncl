# Host-side code generation feeding a real x86_64-linux-musl target build.

let mantle = import "lib.ncl" in
let bootstrap_seed = import "../../../bootstrap/seed.ncl" in
let seed_toolchain = bootstrap_seed.toolchain in
let seed_name = bootstrap_seed.name in
let seed_target = bootstrap_seed.target in

let source_file = fun relative_path input_name digest => mantle.fetchurl {
  url = "file://./%{relative_path}",
  hash = digest,
  name = input_name,
} in

let render_header_source = source_file "tools/render-header.sh" "cross-host-render-header-source" "blake3-rWXN7ox+kB2AaXHCn27WSq58iDNF/n5qccXm87UT1b4=" in
let main_source = source_file "src/main.c" "cross-target-main-source" "blake3-IsNGOJ4N8r2CUHgBRcAW1f3u7Lu6kUcyLOHnapb6rho=" in

let host_tool = {
  name = "cross-host-header-tool",
  builder = "/bin/sh",
  inputs = [render_header_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      tool_source=""
      for candidate in "$NIX_STORE"/*-cross-host-render-header-source; do
        if [ -f "$candidate" ]; then tool_source="$candidate"; break; fi
      done
      if [ -z "$tool_source" ]; then
        echo 'host header tool source was not mounted' >&2
        exit 1
      fi
      $BB mkdir -p "$out/bin" "$out/share/cross-role"
      $BB cp "$tool_source" "$out/bin/render-header"
      $BB chmod 0755 "$out/bin/render-header"
      $BB printf '%s\n' 'host-tool' > "$out/share/cross-role/role"
    "%
  ],
} | mantle.Derivation in

let generated_header = {
  name = "cross-generated-header",
  builder = "/bin/sh",
  inputs = [host_tool],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      tool=""
      for candidate in "$NIX_STORE"/*-cross-host-header-tool; do
        if [ -x "$candidate/bin/render-header" ]; then tool="$candidate"; break; fi
      done
      if [ -z "$tool" ]; then
        echo 'host tool was not mounted' >&2
        exit 1
      fi
      role=$($BB cat "$tool/share/cross-role/role")
      if [ "$role" != 'host-tool' ]; then
        echo "expected host-tool role, got: $role" >&2
        exit 1
      fi
      TOOLS=/tmp/cross-host-tools
      $BB mkdir -p "$TOOLS" "$out/include" "$out/share/cross-role"
      $BB ln -s "$BB" "$TOOLS/cat"
      export PATH="$TOOLS"
      "$tool/bin/render-header" '%{seed_target}' "$out/include/build_target.h"
      $BB printf '%s\n' 'generated-target-input' > "$out/share/cross-role/role"
    "%
  ],
} | mantle.Derivation in

let wrong_role_header = {
  name = "cross-wrong-role-header",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out/include" "$out/share/cross-role"
      $BB printf '%s\n' '#define BUILD_TARGET_TRIPLE "wrong"' > "$out/include/build_target.h"
      $BB printf '%s\n' 'target-artifact' > "$out/share/cross-role/role"
    "%
  ],
} | mantle.Derivation in

let compile_target = fun derivation_name header_input => {
  name = derivation_name,
  builder = "/bin/sh",
  inputs = [main_source, header_input, seed_toolchain],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      main=""
      header=""
      toolchain=""
      for candidate in "$NIX_STORE"/*-cross-target-main-source; do
        if [ -f "$candidate" ]; then main="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-cross-*-header; do
        if [ -f "$candidate/include/build_target.h" ]; then header="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-%{seed_name}; do
        if [ -x "$candidate/bin/%{seed_target}-gcc" ]; then toolchain="$candidate"; break; fi
      done
      if [ -z "$main" ] || [ -z "$header" ] || [ -z "$toolchain" ]; then
        echo 'cross build input was not mounted' >&2
        exit 1
      fi
      role=$($BB cat "$header/share/cross-role/role")
      if [ "$role" != 'generated-target-input' ]; then
        echo "host/target role mismatch: expected generated-target-input, got: $role" >&2
        exit 1
      fi
      GCC="$toolchain/bin/%{seed_target}-gcc"
      "$GCC" -x c -std=c11 -Wall -Wextra -Werror -static -I"$header/include" "$main" -o /tmp/cross-demo
      $BB mkdir -p "$out/bin" "$out/share/cross-role"
      $BB cp /tmp/cross-demo "$out/bin/cross-demo"
      $BB printf '%s\n' '%{seed_target}' > "$out/share/cross-role/target-triple"
      "$out/bin/cross-demo" > "$out/smoke.txt"
      if "$out/bin/cross-demo" extra >/dev/null 2>&1; then
        echo 'target binary accepted an invalid argument' >&2
        exit 1
      fi
    "%
  ],
} | mantle.Derivation in

let target_package = compile_target "cross-musl-target" generated_header in
let role_mismatch = compile_target "cross-role-mismatch" wrong_role_header in

{
  packages = {
    host-tool = host_tool,
    generated-header = generated_header,
    target = target_package,
    role-mismatch = role_mismatch,
  },
  default = { package = "target" },
} | mantle.Project
