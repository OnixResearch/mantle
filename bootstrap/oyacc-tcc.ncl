# Build oyacc 6.6 using tinycc 0.9.27.
#
# oyacc is a small BSD-derived yacc implementation used as the first
# parser generator in the chain, before bison is available.
#
# Adapted from live-bootstrap steps/oyacc-6.6.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in

let oyacc_src = crunch.fetchTarball {
  url = "https://github.com/ibara/yacc/releases/download/oyacc-6.6/oyacc-6.6.tar.gz",
  hash = "sha256-cnz6/3/RGk6t1XsJIrFhzkEEQqaw4rilMWHn+Ay2DUc=",
  name = "oyacc-6.6-src",
} in

{
  name = "oyacc-6.6-tcc",
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

      TCC=$(find_input tinycc-0.9.27)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input oyacc-6.6-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/oyacc-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" oyacc-6.6
      $BB chmod -R u+w oyacc-6.6
      cd oyacc-6.6

      # Compile each source file
      for src in closure error lalr lr0 main mkpar output reader \
                 skeleton symtab verbose warshall; do
        tcc -c -DHAVE_NBTOOL_CONFIG_H=0 "$src.c" -o "$src.o"
      done

      # Link
      tcc -static \
        closure.o error.o lalr.o lr0.o main.o mkpar.o output.o \
        reader.o skeleton.o symtab.o verbose.o warshall.o \
        -o yacc

      # Verify
      ./yacc --help 2>&1 | $BB head -1 || true

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp yacc "$out/bin/yacc"
      $BB chmod 755 "$out/bin/yacc"

      test -x "$out/bin/yacc"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, oyacc_src],
} | crunch.Derivation
