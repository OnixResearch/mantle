# Self-referential package set: libfoo + app.
#
# Demonstrates packages referencing each other within a set.
# mantle builds them in dependency order automatically.
#
# Usage:
#   1. Generate seed: mantle bootstrap -o examples/seed.ncl
#   2. Build: mantle build examples/package-set.ncl -I examples -I builders

let mantle = import "lib.ncl" in
let builders = import "builders/lib.ncl" in
let seed = import "seed.ncl" in
let stdenv = builders.mkStdenv seed in

{
  libfoo = stdenv.mkDerivation {
    pname = "libfoo",
    version = "0.1",
    # Static library and header only: there is no runnable bin/libfoo.
    finish_gates.version.enabled = false,
    buildInputs = [seed.gcc],
    buildPhase = m%"
      cat > foo.c << 'EOF'
      #include <stdio.h>
      void foo() { printf("foo!\n"); }
      EOF
      cat > foo.h << 'EOF'
      void foo();
      EOF
      gcc -c -o foo.o foo.c
      ar rcs libfoo.a foo.o
    "%,
    installPhase = m%"
      mkdir -p $out/lib $out/include
      cp libfoo.a $out/lib/
      cp foo.h $out/include/
    "%,
  },

  app = stdenv.mkDerivation {
    pname = "app",
    version = "0.1",
    buildInputs = [seed.gcc],
    buildPhase = m%"
      cat > main.c << 'EOF'
      #include <stdio.h>
      #include <string.h>
      int main(int argc, char *argv[]) {
          if (argc > 1 && strcmp(argv[1], "--version") == 0) {
              printf("app 0.1\n");
              return 0;
          }
          printf("app works!\n");
          return 0;
      }
      EOF
      gcc -o app main.c
    "%,
    installPhase = m%"
      mkdir -p $out/bin
      cp app $out/bin/
    "%,
  },
}
