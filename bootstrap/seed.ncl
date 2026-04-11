# Bootstrap seed provider.
#
# The public seed contract is a normalized toolchain derivation with a
# stable layout: target-prefixed binutils in `bin/`, a target sysroot at
# `<target>/`, and libgcc available under `<target>/lib/`.
#
# Today that normalized toolchain is built by repackaging the fetched
# musl.cc native tarball. Later changes can swap the raw provider without
# touching the rest of the bootstrap chain.

let crunch = import "lib.ncl" in

let raw_name = "musl-gcc-raw" in
let seed_name = "musl-seed-toolchain" in
let seed_target = "x86_64-linux-musl" in
let seed_dynamic_linker = "ld-musl-x86_64.so.1" in
let raw_toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = raw_name,
} in
let seed_toolchain = {
  name = seed_name,
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      RAW=""
      for d in $NIX_STORE/*-%{raw_name}; do
        if [ -d "$d/bin" ]; then RAW="$d"; break; fi
      done
      if [ -z "$RAW" ]; then
        echo "ERROR: raw seed toolchain not found" >&2
        exit 1
      fi

      $BB mkdir -p "$out"
      $BB cp -r "$RAW"/. "$out"
      $BB chmod -R u+w "$out"

      $BB mkdir -p "$out/%{seed_target}/lib"
      $BB ln -sfn "$out/include" "$out/%{seed_target}/include"

      for tool in addr2line ar as c++filt dwp elfedit gprof ld ld.bfd ld.gold \
                  nm objcopy objdump ranlib readelf size strings strip; do
        if [ -x "$out/bin/$tool" ]; then
          $BB ln -sfn "$out/bin/$tool" "$out/bin/%{seed_target}-$tool"
        fi
      done

      for lib in libgcc_s.so libgcc_s.so.1 libc.so; do
        if [ -f "$out/lib/$lib" ]; then
          $BB ln -sfn "$out/lib/$lib" "$out/%{seed_target}/lib/$lib"
        fi
      done

      test -x "$out/bin/%{seed_target}-gcc"
      test -x "$out/bin/%{seed_target}-ar"
      test -x "$out/bin/%{seed_target}-ld"
      test -f "$out/%{seed_target}/lib/libgcc_s.so.1"
      test -d "$out/%{seed_target}/include/linux"
    "%,
  ],
  inputs = [raw_toolchain],
} | crunch.Derivation in

{
  name = seed_name,
  target = seed_target,
  dynamic_linker = seed_dynamic_linker,
  toolchain = seed_toolchain,
}
