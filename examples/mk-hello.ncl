# Build a C hello world using mkDerivation + stdenv.
#
# Usage:
#   1. Generate seed: mantle bootstrap -o examples/seed.ncl
#   2. Build: mantle build examples/mk-hello.ncl -I examples -I builders

let mantle = import "lib.ncl" in
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
    #include <string.h>
    int main(int argc, char *argv[]) {
        if (argc > 1 && strcmp(argv[1], "--version") == 0) {
            printf("hello 1.0\n");
            return 0;
        }
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
