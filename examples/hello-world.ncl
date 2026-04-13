# Build a C hello world program with either bootstrap seed mode.
#
# Usage:
#   1. Generate seed: crunch bootstrap -o seed.ncl
#      or: crunch bootstrap --fetch --store /tmp/crunch-store -o seed.ncl
#   2. Build from a directory containing both files:
#      crunch build hello-world.ncl -I . --store /tmp/crunch-store

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "hello-world",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      CC=""
      if [ -x "%{seed.gcc}/bin/gcc" ]; then
        CC="%{seed.gcc}/bin/gcc"
      elif [ -x "%{seed.gcc}/bin/x86_64-linux-musl-gcc" ]; then
        CC="%{seed.gcc}/bin/x86_64-linux-musl-gcc"
      fi
      if [ -z "$CC" ]; then
        echo "ERROR: no usable C compiler in %{seed.gcc}" >&2
        exit 1
      fi

      $BB mkdir -p "$out/bin"
      $BB cat > hello.c << 'EOF'
      #include <stdio.h>
      int main() {
          printf("Hello from crunch!\n");
          return 0;
      }
      EOF

      "$CC" -o "$out/bin/hello" hello.c
    "%,
  ],
  inputs = [seed.gcc],
} | crunch.Derivation
