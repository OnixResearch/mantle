# Build bzip2 1.0.8 using tinycc 0.9.27.
#
# bzip2 is needed to decompress .tar.bz2 source archives later in the
# chain. Only bzip2/bunzip2 binaries are installed; the library is not
# needed by downstream stages.
#
# Adapted from live-bootstrap steps/bzip2-1.0.8.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in
let patch = import "patch-tcc.ncl" in

let bzip2_src = crunch.fetchTarball {
  url = "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
  hash = "sha256-Uvi4JZPPERK3gym4yoaeTEJwKXF5brBAEN7GgF+iF6g=",
  name = "bzip2-1.0.8-src",
} in

{
  name = "bzip2-1.0.8-tcc",
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
      SRC=$(find_input bzip2-1.0.8-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bzip2-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bzip2-1.0.8
      $BB chmod -R u+w bzip2-1.0.8
      cd bzip2-1.0.8

      CFLAGS="-D_FILE_OFFSET_BITS=64"

      # Compile library objects
      for src in blocksort huffman crctable randtable compress decompress bzlib; do
        tcc -c $CFLAGS $src.c -o $src.o
      done

      # Build static library
      tcc -ar cr libbz2.a \
        blocksort.o huffman.o crctable.o randtable.o compress.o decompress.o bzlib.o

      # Compile and link bzip2
      tcc -c $CFLAGS bzip2.c -o bzip2.o
      tcc -static bzip2.o -L. -lbz2 -o bzip2

      # Compile and link bzip2recover
      tcc -c $CFLAGS bzip2recover.c -o bzip2recover.o
      tcc -static bzip2recover.o -o bzip2recover

      # Verify
      ./bzip2 --help 2>&1 | $BB head -1

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp bzip2 "$out/bin/bzip2"
      $BB cp bzip2recover "$out/bin/bzip2recover"
      $BB chmod 755 "$out/bin/bzip2" "$out/bin/bzip2recover"

      # bunzip2 and bzcat are just bzip2 invoked under different names
      cd "$out/bin"
      $BB ln -s bzip2 bunzip2
      $BB ln -s bzip2 bzcat

      test -x "$out/bin/bzip2"
      test -x "$out/bin/bunzip2"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, patch, bzip2_src],
} | crunch.Derivation
