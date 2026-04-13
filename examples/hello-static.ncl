# Build a statically linked C hello world.
# Static linking avoids the ld.so interpreter path issue.

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "hello-static",
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
          printf("Hello from crunch (static)!\n");
          return 0;
      }
      EOF

      "$CC" -static -o "$out/bin/hello" hello.c
    "%,
  ],
  inputs = [seed.gcc],
} | crunch.Derivation
