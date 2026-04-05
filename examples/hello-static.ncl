# Build a statically linked C hello world.
# Static linking avoids the ld.so interpreter path issue.

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "hello-static",
  builder = "%{seed.bash}/bin/bash",
  args = [
    "-c",
    m%"
      set -e
      export PATH="%{seed.coreutils}/bin:%{seed.gcc}/bin"
      mkdir -p $out/bin

      cat > hello.c << 'EOF'
      #include <stdio.h>
      int main() {
          printf("Hello from crunch (static)!\n");
          return 0;
      }
      EOF

      gcc -static -o $out/bin/hello hello.c
    "%,
  ],
  inputs = [
    seed.bash,
    seed.coreutils,
    seed.gcc,
  ],
} | crunch.Derivation
