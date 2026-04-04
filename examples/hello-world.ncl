# Build a C hello world program using the seed toolchain.
#
# Usage:
#   1. Generate seed: crunch bootstrap -o examples/seed.ncl
#   2. Build: crunch build examples/hello-world.ncl -I examples

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "hello-world",
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
          printf("Hello from crunch!\n");
          return 0;
      }
      EOF

      gcc -o $out/bin/hello hello.c
    "%,
  ],
  inputs = [
    seed.bash,
    seed.coreutils,
    seed.gcc,
  ],
} | crunch.Derivation
