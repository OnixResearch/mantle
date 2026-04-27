# Rebuild bzip2 1.0.8 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/bzip2-1.0.8 (musl rebuild).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in

let bzip2_src = crunch.fetchTarball {
  url = "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
  hash = "sha256-q1oDF27hBtPw+pDjgdpHjdrkBZGBU8yiSOaCzQxKImk=",
  name = "bzip2-1.0.8-src",
} in

{
  name = "bzip2-1.0.8-musl",
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

      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input bzip2-1.0.8-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bzip2-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bzip2-1.0.8
      $BB chmod -R u+w bzip2-1.0.8
      cd bzip2-1.0.8

      CFLAGS="-I$MUSL/include -D_FILE_OFFSET_BITS=64"

      for src in blocksort huffman crctable randtable compress decompress bzlib; do
        tcc -c $CFLAGS $src.c -o $src.o
      done
      tcc -ar cr libbz2.a blocksort.o huffman.o crctable.o randtable.o compress.o decompress.o bzlib.o

      tcc -c $CFLAGS bzip2.c -o bzip2.o
      tcc -static bzip2.o -L. -lbz2 -o bzip2

      tcc -c $CFLAGS bzip2recover.c -o bzip2recover.o
      tcc -static bzip2recover.o -o bzip2recover

      ./bzip2 --help 2>&1 | $BB head -1

      $BB mkdir -p "$out/bin"
      $BB cp bzip2 "$out/bin/bzip2"
      $BB cp bzip2recover "$out/bin/bzip2recover"
      $BB chmod 755 "$out/bin/bzip2" "$out/bin/bzip2recover"
      cd "$out/bin"
      $BB ln -s bzip2 bunzip2
      $BB ln -s bzip2 bzcat

      test -x "$out/bin/bzip2"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, bzip2_src],
} | crunch.Derivation
