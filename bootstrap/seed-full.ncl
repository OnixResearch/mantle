# Full-source bootstrap seed provider.
#
# Exposes the same normalized seed contract as seed-legacy.ncl but
# backed by the from-source chain: hex0 → stage0-posix → mes → tcc →
# gcc version ladder → musl + binutils.
#
# TODO: once the GCC ladder placeholder derivations are implemented,
# this file should normalize the gcc-10 + musl-1.2 + binutils-2.41
# output into the seed contract layout.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in
let musl = import "musl-full.ncl" in
let binutils = import "binutils-full.ncl" in

let seed_name = "full-source-seed-toolchain" in
let seed_target = "x86_64-linux-musl" in
let seed_dynamic_linker = "ld-musl-x86_64.so.1" in

let seed_toolchain = {
  name = seed_name,
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      GCC=$(find_input gcc-10.5.0)
      MUSL=$(find_input musl-1.2.5-full)
      BINUTILS=$(find_input binutils-2.41-full)

      # Normalize into the seed contract layout:
      # - target-prefixed binutils in bin/
      # - sysroot at <target>/
      # - provider metadata in share/crunch-bootstrap/

      $BB mkdir -p "$out/bin" "$out/lib" "$out/libexec"
      $BB mkdir -p "$out/%{seed_target}/include" "$out/%{seed_target}/lib"
      $BB mkdir -p "$out/share/crunch-bootstrap"

      # TODO: copy and normalize GCC, musl, binutils outputs
      # into the seed contract layout. This mirrors what
      # seed-legacy.ncl does for the musl.cc tarball.

      echo "ERROR: seed-full.ncl normalization not yet implemented" >&2
      exit 1
    "%,
  ],
  inputs = [gcc10, musl, binutils],
} | crunch.Derivation in

{
  name = seed_name,
  target = seed_target,
  dynamic_linker = seed_dynamic_linker,
  toolchain = seed_toolchain,
  provider = {
    id = "full-source-v1",
    summary = "Full-source seed built from hex0 through GCC version ladder",
    raw = {
      name = "from-source",
      url = "n/a",
      hash = "n/a",
    },
    retained_tools = [
      "x86_64-linux-musl-gcc",
      "x86_64-linux-musl-g++",
      "x86_64-linux-musl-ar",
      "x86_64-linux-musl-as",
      "x86_64-linux-musl-ld",
      "x86_64-linux-musl-nm",
      "x86_64-linux-musl-objcopy",
      "x86_64-linux-musl-objdump",
      "x86_64-linux-musl-ranlib",
      "x86_64-linux-musl-readelf",
      "x86_64-linux-musl-strip",
    ],
    dropped_components = [],
    notes = [
      "Every tool built from source starting from a 229-byte hex0 seed.",
      "Chain: hex0 → stage0-posix → mes → tcc → gcc-4.0 → gcc-4.7 → gcc-10 → musl + binutils.",
    ],
  },
}
