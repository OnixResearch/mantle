# Build a C hello world using mkDerivation + stdenv.
#
# Usage:
#   1. Generate seed: crunch bootstrap -o examples/seed.ncl
#   2. Build: crunch build examples/mk-hello.ncl -I examples -I builders

let crunch = import "lib.ncl" in
let builders = import "builders/lib.ncl" in
let seed = import "seed.ncl" in
let stdenv = builders.mkStdenv seed in

stdenv.mkDerivation {
  pname = "hello",
  version = "1.0",
  buildInputs = [seed.gcc],
  buildPhase = m%"
    cat > hello.c << 'EOF'
    #include <stdio.h>
    int main() {
        printf("Hello from mkDerivation!\n");
        return 0;
    }
    EOF
    gcc -o hello hello.c
  "%,
  installPhase = m%"
    mkdir -p $out/bin
    cp hello $out/bin/
  "%,
}
