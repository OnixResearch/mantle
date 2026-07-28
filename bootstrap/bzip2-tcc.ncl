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
let sed = import "sed-tcc.ncl" in

let bzip2_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/bzip2-1.0.8/sources
  # first-consumer: bootstrap/bzip2-tcc.ncl
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

      $BB cat > utime.h << 'EOF'
#ifndef CRUNCH_BOOTSTRAP_UTIME_H
#define CRUNCH_BOOTSTRAP_UTIME_H
struct utimbuf { long actime; long modtime; };
int utime(const char *filename, const struct utimbuf *times);
#endif
EOF
      # Mes libc on this handoff does not provide fchmod/fchown symbols.
      # bzip2's bootstrap contract is executable compression/decompression;
      # metadata preservation is outside this early tool boundary.
      $BB sed -i 's/retVal = utime ( dstName, \&uTimBuf );/retVal = 0;/' bzip2.c
      $BB sed -i 's/retVal = fchmod ( fd, fileMetaInfo.st_mode );/retVal = 0;/' bzip2.c
      $BB sed -i 's/(void) fchown ( fd, fileMetaInfo.st_uid, fileMetaInfo.st_gid );/(void)0;/' bzip2.c
      CFLAGS="-D_FILE_OFFSET_BITS=64 -I."

      # Compile library objects. This part installs only executables, not
      # libbz2.a, so link the objects directly instead of using `tcc -ar`.
      # The Mes-built TinyCC ar path can segfault after emitting only
      # `tcc: ar: can't open file`; avoiding the unused archive keeps this
      # part bound to the output contract live-bootstrap needs.
      BZLIB_OBJS="blocksort.o huffman.o crctable.o randtable.o compress.o decompress.o bzlib.o"
      for src in blocksort huffman crctable randtable compress decompress bzlib; do
        tcc -c $CFLAGS $src.c -o $src.o
        $BB chmod u+rw $src.o
      done

      # Compile and link bzip2
      tcc -c $CFLAGS bzip2.c -o bzip2.o
      $BB chmod u+rw bzip2.o
      tcc -static bzip2.o $BZLIB_OBJS -o bzip2
      $BB chmod 755 bzip2

      # Compile and link bzip2recover
      tcc -c $CFLAGS bzip2recover.c -o bzip2recover.o
      $BB chmod u+rw bzip2recover.o
      tcc -static bzip2recover.o -o bzip2recover
      $BB chmod 755 bzip2recover

      # Verify the built binary before installation. Upstream pass1 runs
      # `bzip2 --help`; Crunch also installs bzip2recover and bzcat, so the
      # installed contract is checked again after copying.
      ./bzip2 --help 2>&1 | $BB head -1
      test -x ./bzip2
      test -x ./bzip2recover

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
      test -x "$out/bin/bzcat"
      test -x "$out/bin/bzip2recover"

      "$out/bin/bzip2" --help 2>&1 | $BB head -1
      "$out/bin/bunzip2" --help 2>&1 | $BB head -1
      "$out/bin/bzcat" --help 2>&1 | $BB head -1
      "$out/bin/bzip2recover" 2>&1 | $BB head -1
    "%,
  ],
  inputs = [stage0, mes, tcc, make, patch, sed, bzip2_src],
} | crunch.Derivation
