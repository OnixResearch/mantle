# Rebuild bzip2 1.0.8 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/bzip2-1.0.8 (musl rebuild).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26 = import "tinycc-mes.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in

let bzip2_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/bzip2-1.0.8/sources (musl rebuild entry)
  # first-consumer: bootstrap/bzip2-1.0.8-musl.ncl
  url = "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
  hash = "sha256-Uvi4JZPPERK3gym4yoaeTEJwKXF5brBAEN7GgF+iF6g=",
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

      TCC26=$(find_input tinycc-0.9.26)
      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input bzip2-1.0.8-src)

      export PATH="$TCC26/bin:$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bzip2-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bzip2-1.0.8
      $BB chmod -R u+w bzip2-1.0.8
      cd bzip2-1.0.8

      CFLAGS="-I$MUSL/include -D_FILE_OFFSET_BITS=64"

      $BB cat > crunch-start.c <<'EOF'
int main(int argc, char **argv, char **envp);
void _start(void) { char *argv[1]; argv[0] = 0; main(0, argv, argv); for (;;) {} }
EOF
      tcc -c $CFLAGS crunch-start.c -o crunch-start.o

      for src in blocksort huffman crctable randtable compress decompress bzlib; do
        echo "-> $src.c"
        tcc -c $CFLAGS $src.c -o $src.o
        test -s $src.o
        echo "<- $src.o"
      done

      # The Mes-linked tcc-musl-v2 bridge can still hit its fragile library
      # search/diagnostic path for -L/-l and implicit startup/runtime lookup.
      # Keep this stage explicit: compile objects, then link with declared musl
      # crt/libc and installed libtcc1 directly.
      echo "-> bzip2.c"
      tcc -c $CFLAGS bzip2.c -o bzip2.o
      test -s bzip2.o
      echo "<- bzip2.o"
      echo "-> bzip2 link"
      tcc -v -nostdlib -static -o bzip2 \
        crunch-start.o bzip2.o \
        blocksort.o huffman.o crctable.o randtable.o compress.o decompress.o bzlib.o \
        "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
      echo "<- bzip2"

      echo "-> bzip2recover.c"
      tcc -c $CFLAGS bzip2recover.c -o bzip2recover.o
      test -s bzip2recover.o
      echo "<- bzip2recover.o"
      echo "-> bzip2recover link"
      tcc -v -nostdlib -static -o bzip2recover \
        crunch-start.o bzip2recover.o \
        "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
      echo "<- bzip2recover"

      $BB chmod 755 ./bzip2 ./bzip2recover
      test -x ./bzip2
      test -x ./bzip2recover
      ./bzip2 --help 2>&1 | $BB head -1

      $BB mkdir -p "$out/bin"
      $BB cp bzip2 "$out/bin/bzip2"
      $BB cp bzip2recover "$out/bin/bzip2recover"
      $BB chmod 755 "$out/bin/bzip2" "$out/bin/bzip2recover"
      cd "$out/bin"
      $BB ln -s bzip2 bunzip2
      $BB ln -s bzip2 bzcat

      test -x "$out/bin/bzip2"
      test -x "$out/bin/bunzip2"
      test -x "$out/bin/bzcat"
      test -x "$out/bin/bzip2recover"

      "$out/bin/bzip2" --help 2>&1 | $BB head -1
      "$out/bin/bunzip2" --help 2>&1 | $BB head -1
      "$out/bin/bzcat" --help 2>&1 | $BB head -1
      "$out/bin/bzip2recover" 2>&1 | $BB head -1
    "%,
  ],
  inputs = [stage0, tcc26, tcc, musl, make, bzip2_src],
} | crunch.Derivation
