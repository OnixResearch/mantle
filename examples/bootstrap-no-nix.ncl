# Bootstrap a C toolchain without Nix.
#
# Fetches a statically-linked musl-gcc from musl.cc, then uses it
# to compile a C program. Zero Nix dependencies at runtime.
#
# Usage:
#   crunch build examples/bootstrap-no-nix.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

{
  name = "hello-no-nix",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      # Set up busybox applet symlinks so mkdir/cp/cat/etc work as commands.
      # /bin/busybox is the same busybox-static binary mounted at /bin/sh,
      # but with the correct argv[0] for applet dispatch.
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      # Find the musl-gcc toolchain.
      TOOLCHAIN=""
      for d in /nix/store/*-musl-gcc; do
        if [ -d "$d" ]; then TOOLCHAIN="$d"; break; fi
      done
      if [ -z "$TOOLCHAIN" ]; then
        echo "ERROR: musl-gcc not found" >&2; exit 1
      fi

      GCC="$TOOLCHAIN/bin/x86_64-linux-musl-gcc"

      # Write source
      cat > /tmp/hello.c << 'CEOF'
#include <stdio.h>
int main() {
    printf("Hello from crunch — no Nix involved!\n");
    return 0;
}
CEOF

      # Compile statically with the fetched musl-gcc
      "$GCC" -static -o /tmp/hello /tmp/hello.c

      # Install
      mkdir -p $out/bin
      cp /tmp/hello $out/bin/hello
    "%,
  ],
  inputs = [toolchain],
} | crunch.Derivation
