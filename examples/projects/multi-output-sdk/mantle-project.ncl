# C SDK with runtime, development, documentation, and debug outputs.

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

let sdk_header = source_file "include/mantle_sdk.h" "sdk-header" "blake3-tDTORuAlC+Qc6/sNquuSheSNaGU7BVdOoqK1SJL/jic=" in
let sdk_source = source_file "src/mantle_sdk.c" "sdk-source" "blake3-e5xSXVEhZPeYU/BTPqe1gxApq9Tx28aFbCP/vRY+yR4=" in
let cli_source = source_file "src/main.c" "sdk-cli-source" "blake3-Hx8aCv/4+5OmK3Up8PhoRCpHcBUxQmGi59HpZF0WQr4=" in
let test_source = source_file "tests/test_sdk.c" "sdk-test-source" "blake3-dIAIPnf1qUapsB73ilx2+5oHD8IV7x2jGhmADKaRISM=" in
let guide_source = source_file "docs/README.md" "sdk-guide" "blake3-mm+TNRSE4QE3lSx4n63288+4qkRzY2iIGfCae/uO14c=" in
let man_source = source_file "docs/mantle-sdk.1" "sdk-man-page" "blake3-KEMZaMTz3u7YeRn90x6Dwss8/W7iXASaEjBLm+7FxFw=" in

let sdk_source_tree = {
  name = "multi-output-sdk-source",
  builder = "/bin/sh",
  inputs = [sdk_header, sdk_source, cli_source, test_source, guide_source, man_source],
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
          echo "fixed SDK source input not mounted: $input_name" >&2
          exit 1
        fi
        $BB cp "$source_path" "$destination"
      }

      $BB mkdir -p "$out/include" "$out/src" "$out/tests" "$out/docs"
      copy_input sdk-header "$out/include/mantle_sdk.h"
      copy_input sdk-source "$out/src/mantle_sdk.c"
      copy_input sdk-cli-source "$out/src/main.c"
      copy_input sdk-test-source "$out/tests/test_sdk.c"
      copy_input sdk-guide "$out/docs/README.md"
      copy_input sdk-man-page "$out/docs/mantle-sdk.1"
    "%
  ],
} | mantle.Derivation in

let sdk_package = {
  name = "mantle-sdk",
  outputs = ["out", "dev", "doc", "debug"],
  builder = "/bin/sh",
  inputs = [sdk_source_tree, seed_toolchain],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      OPTIMIZATION_LEVEL=2
      C_STANDARD=c11
      WORK=/tmp/mantle-sdk
      $BB mkdir -p "$WORK/build"

      source_input=""
      for candidate in "$NIX_STORE"/*-multi-output-sdk-source; do
        if [ -f "$candidate/include/mantle_sdk.h" ]; then
          source_input="$candidate"
          break
        fi
      done
      toolchain_input=""
      for candidate in "$NIX_STORE"/*-%{seed_name}; do
        if [ -x "$candidate/bin/%{seed_target}-gcc" ]; then
          toolchain_input="$candidate"
          break
        fi
      done
      if [ -z "$source_input" ] || [ -z "$toolchain_input" ]; then
        echo 'SDK source or bootstrap toolchain was not mounted' >&2
        exit 1
      fi

      $BB cp -r "$source_input/." "$WORK/"
      GCC="$toolchain_input/bin/%{seed_target}-gcc"
      AR="$toolchain_input/bin/%{seed_target}-ar"
      CFLAGS="-O$OPTIMIZATION_LEVEL -std=$C_STANDARD -Wall -Wextra -Werror"

      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/src/mantle_sdk.c" -o "$WORK/build/mantle_sdk.o"
      "$AR" rcs "$WORK/build/libmantle_sdk.a" "$WORK/build/mantle_sdk.o"
      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/tests/test_sdk.c" -o "$WORK/build/test_sdk.o"
      "$GCC" -static $CFLAGS "$WORK/build/test_sdk.o" "$WORK/build/libmantle_sdk.a" -o "$WORK/build/test_sdk"
      "$WORK/build/test_sdk"
      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/src/main.c" -o "$WORK/build/main.o"
      "$GCC" -static -s $CFLAGS "$WORK/build/main.o" "$WORK/build/libmantle_sdk.a" -o "$WORK/build/mantle-sdk-greet"
      "$GCC" -I"$WORK/include" -g $CFLAGS -c "$WORK/src/main.c" -o "$WORK/build/main-debug.o"
      "$GCC" -static -g $CFLAGS "$WORK/build/main-debug.o" "$WORK/build/libmantle_sdk.a" -o "$WORK/build/mantle-sdk-greet.debug"

      $BB mkdir -p "$out/bin" "$dev/include" "$dev/lib/pkgconfig" "$doc/share/doc/mantle-sdk" "$doc/share/man/man1" "$debug/lib/debug"
      $BB cp "$WORK/build/mantle-sdk-greet" "$out/bin/mantle-sdk-greet"
      $BB cp "$WORK/include/mantle_sdk.h" "$dev/include/mantle_sdk.h"
      $BB cp "$WORK/build/libmantle_sdk.a" "$dev/lib/libmantle_sdk.a"
      $BB cp "$WORK/docs/README.md" "$doc/share/doc/mantle-sdk/README.md"
      $BB cp "$WORK/docs/mantle-sdk.1" "$doc/share/man/man1/mantle-sdk-greet.1"
      $BB cp "$WORK/build/mantle-sdk-greet.debug" "$debug/lib/debug/mantle-sdk-greet"
      $BB cat > "$dev/lib/pkgconfig/mantle-sdk.pc" << EOF
prefix=$dev
includedir=\${prefix}/include
libdir=\${prefix}/lib

Name: mantle-sdk
Description: bounded greeting SDK
Version: 1.0.0
Cflags: -I\${includedir}
Libs: -L\${libdir} -lmantle_sdk
EOF
      "$out/bin/mantle-sdk-greet" Mantle > "$out/smoke.txt"
    "%
  ],
} | mantle.Derivation in

let runtime_consumer = {
  name = "mantle-sdk-runtime-consumer",
  builder = "/bin/sh",
  inputs = [mantle.select sdk_package "out"],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      runtime_input=""
      for candidate in "$NIX_STORE"/*; do
        if [ -x "$candidate/bin/mantle-sdk-greet" ]; then
          runtime_input="$candidate"
          break
        fi
      done
      if [ -z "$runtime_input" ]; then
        echo 'selected SDK runtime output was not mounted' >&2
        exit 1
      fi
      if [ -e "$runtime_input/include/mantle_sdk.h" ]; then
        echo 'development header leaked into runtime output' >&2
        exit 1
      fi
      output=$($runtime_input/bin/mantle-sdk-greet Runtime)
      if [ "$output" != 'Hello, Runtime!' ]; then
        echo "unexpected runtime consumer output: $output" >&2
        exit 1
      fi
      if "$runtime_input/bin/mantle-sdk-greet" first second; then
        echo 'runtime CLI accepted too many arguments' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'multi-output-sdk-runtime-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

let development_consumer = {
  name = "mantle-sdk-development-consumer",
  builder = "/bin/sh",
  inputs = [mantle.select sdk_package "dev", seed_toolchain],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      development_input=""
      for candidate in "$NIX_STORE"/*; do
        if [ -f "$candidate/include/mantle_sdk.h" ] && [ -f "$candidate/lib/libmantle_sdk.a" ]; then
          development_input="$candidate"
          break
        fi
      done
      toolchain_input=""
      for candidate in "$NIX_STORE"/*-%{seed_name}; do
        if [ -x "$candidate/bin/%{seed_target}-gcc" ]; then
          toolchain_input="$candidate"
          break
        fi
      done
      if [ -z "$development_input" ] || [ -z "$toolchain_input" ]; then
        echo 'selected SDK development output or compiler was not mounted' >&2
        exit 1
      fi
      if [ -e "$development_input/bin/mantle-sdk-greet" ]; then
        echo 'runtime binary leaked into development output' >&2
        exit 1
      fi

      $BB cat > /tmp/consumer.c << 'EOF'
#include <mantle_sdk.h>
#include <stdio.h>

enum { OUTPUT_CAPACITY_BYTES = 64 };

int main(void) {
    char output[OUTPUT_CAPACITY_BYTES];
    if (mantle_sdk_format("Developer", output, sizeof(output)) != MANTLE_SDK_STATUS_OK) {
        return 1;
    }
    puts(output);
    return 0;
}
EOF
      GCC="$toolchain_input/bin/%{seed_target}-gcc"
      "$GCC" -static -std=c11 -Wall -Wextra -Werror -I"$development_input/include" /tmp/consumer.c "$development_input/lib/libmantle_sdk.a" -o /tmp/consumer
      output=$(/tmp/consumer)
      if [ "$output" != 'Hello, Developer!' ]; then
        echo "unexpected development consumer output: $output" >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'multi-output-sdk-development-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    source = sdk_source_tree,
    sdk = sdk_package,
    runtime-consumer = runtime_consumer,
    development-consumer = development_consumer,
  },
  checks = {
    runtime = runtime_consumer,
    development = development_consumer,
  },
  default = { package = "sdk" },
} | mantle.Project
