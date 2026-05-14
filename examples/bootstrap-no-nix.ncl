# Bootstrap a C toolchain without Nix.
#
# Uses the shared reduced bootstrap seed provider from bootstrap/seed.ncl,
# then compiles a C program. Zero Nix dependencies at runtime.
#
# Usage:
#   mantle build examples/bootstrap-no-nix.ncl -I lib --store /tmp/mantle-store

let mantle = import "lib.ncl" in
let seed = import "../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in

{
  name = "hello-no-nix",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      TOOLCHAIN=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -d "$d" ]; then TOOLCHAIN="$d"; break; fi
      done
      if [ -z "$TOOLCHAIN" ]; then
        echo "ERROR: bootstrap seed toolchain not found" >&2; exit 1
      fi

      GCC="$TOOLCHAIN/bin/%{seed_target}-gcc"

      cat > /tmp/hello.c << 'CEOF'
#include <stdio.h>
int main() {
    printf("Hello from mantle — no Nix involved!\n");
    return 0;
}
CEOF

      "$GCC" -static -o /tmp/hello /tmp/hello.c

      mkdir -p $out/bin
      cp /tmp/hello $out/bin/hello
    "%,
  ],
  inputs = [toolchain],
} | mantle.Derivation
