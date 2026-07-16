# C library + CLI project using fixed local sources and Mantle's bootstrap toolchain.

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

let greet_h = source_file "include/greet.h" "c-project-greet-h" "blake3-Njh6By5BgMxvNEN4o6GtfilQ+rlFXTvjoV3cZsTVjns=" in
let greet_c = source_file "src/greet.c" "c-project-greet-c" "blake3-qCbv3LyXzswFPyvc/9g2IAydvyT2Zb0//okzz/2j7yk=" in
let main_c = source_file "src/main.c" "c-project-main-c" "blake3-vXkBi4/Cx9mn+9/XPdEsi99plTaK1UWA7BF6k3MFG9c=" in
let test_c = source_file "tests/test_greet.c" "c-project-test-c" "blake3-MwgPop/kwK1sFMbjqcuzq4hweFb5xr7twDMGTuM0h+Q=" in
let makefile = source_file "Makefile" "c-project-makefile" "blake3-8U8Z9Wu+2Tq59sdwwBd2FGaqBogIPQDau+K66K0kD4s=" in

let source_tree = {
  name = "c-library-cli-source",
  builder = "/bin/sh",
  inputs = [greet_h, greet_c, main_c, test_c, makefile],
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
          echo "fixed source input not mounted: $input_name" >&2
          exit 1
        fi
        $BB cp "$source_path" "$destination"
      }

      $BB mkdir -p "$out/include" "$out/src" "$out/tests"
      copy_input c-project-greet-h "$out/include/greet.h"
      copy_input c-project-greet-c "$out/src/greet.c"
      copy_input c-project-main-c "$out/src/main.c"
      copy_input c-project-test-c "$out/tests/test_greet.c"
      copy_input c-project-makefile "$out/Makefile"
    "%
  ],
} | mantle.Derivation in

let greet_package = {
  name = "c-library-cli",
  builder = "/bin/sh",
  inputs = [source_tree, seed_toolchain],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      OPTIMIZATION_LEVEL=2
      C_STANDARD=c11
      WORK=/tmp/c-library-cli
      TOOLS=/tmp/tools
      $BB mkdir -p "$WORK/build" "$TOOLS"
      for command in cat chmod cp dirname echo ln mkdir printf rm; do
        $BB ln -sf "$BB" "$TOOLS/$command"
      done
      export PATH="$TOOLS"

      source_input=""
      for candidate in "$NIX_STORE"/*-c-library-cli-source; do
        if [ -f "$candidate/Makefile" ]; then
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
        echo 'C source or bootstrap toolchain was not mounted' >&2
        exit 1
      fi

      $BB cp -r "$source_input/." "$WORK/"
      GCC="$toolchain_input/bin/%{seed_target}-gcc"
      AR="$toolchain_input/bin/%{seed_target}-ar"
      CFLAGS="-O$OPTIMIZATION_LEVEL -std=$C_STANDARD -Wall -Wextra -Werror"

      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/src/greet.c" -o "$WORK/build/greet.o"
      "$AR" rcs "$WORK/build/libgreet.a" "$WORK/build/greet.o"
      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/src/main.c" -o "$WORK/build/main.o"
      "$GCC" -static $CFLAGS "$WORK/build/main.o" "$WORK/build/libgreet.a" -o "$WORK/build/greet"
      "$GCC" -I"$WORK/include" $CFLAGS -c "$WORK/tests/test_greet.c" -o "$WORK/build/test_greet.o"
      "$GCC" -static $CFLAGS "$WORK/build/test_greet.o" "$WORK/build/libgreet.a" -o "$WORK/build/test_greet"
      "$WORK/build/test_greet"

      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
      $BB cp "$WORK/build/greet" "$out/bin/greet"
      $BB cp "$WORK/build/libgreet.a" "$out/lib/libgreet.a"
      $BB cp "$WORK/include/greet.h" "$out/include/greet.h"
      "$out/bin/greet" Mantle > "$out/smoke.txt"
      if "$out/bin/greet" first second >/dev/null 2>&1; then
        echo 'greet accepted too many arguments' >&2
        exit 1
      fi
    "%
  ],
} | mantle.Derivation in

let greet_check = {
  name = "c-library-cli-check",
  builder = "/bin/sh",
  inputs = [greet_package],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      package_input=""
      for candidate in "$NIX_STORE"/*-c-library-cli; do
        if [ -x "$candidate/bin/greet" ]; then
          package_input="$candidate"
          break
        fi
      done
      if [ -z "$package_input" ]; then
        echo 'C package was not mounted' >&2
        exit 1
      fi
      output=$($package_input/bin/greet Mantle)
      if [ "$output" != 'Hello, Mantle!' ]; then
        echo "unexpected C CLI output: $output" >&2
        exit 1
      fi
      if "$package_input/bin/greet" first second; then
        echo 'C CLI negative invocation unexpectedly succeeded' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'c-library-cli-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    source = source_tree,
    greet = greet_package,
  },
  checks = { test-greet = greet_check },
  default = { package = "greet" },
} | mantle.Project
