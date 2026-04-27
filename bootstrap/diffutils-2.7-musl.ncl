# Build diffutils 2.7 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/diffutils-2.7.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let diffutils_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/diffutils/diffutils-2.7.tar.gz",
  hash = "sha256-1fJInEBWoxUo462krazCPUmFMrCvGpgPL3YVgWKxOdY=",
  name = "diffutils-2.7-src",
} in

{
  name = "diffutils-2.7-musl",
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
      SED=$(find_input sed-4.0.9-musl)
      SRC=$(find_input diffutils-2.7-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/diffutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" diffutils-2.7
      $BB chmod -R u+w diffutils-2.7
      cd diffutils-2.7

      : > config.h

      CFLAGS="-I. -I$MUSL/include -DSTDC_HEADERS -DHAVE_UNISTD_H -DHAVE_STRING_H -DHAVE_FCNTL_H -DHAVE_SYS_TYPES_H -DHAVE_STDLIB_H -DDIRENT -DHAVE_LIMITS_H"

      for src in diff diff3 analyze cmpbuf context dir ed ifdef io normal \
                 side util error getopt getopt1 regex version; do
        tcc -c $CFLAGS $src.c -o $src.o 2>/dev/null || true
      done

      OBJS=""
      for f in *.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -o diff
      tcc -static -I. -I"$MUSL/include" $CFLAGS cmp.c error.o getopt.o getopt1.o version.o -o cmp 2>/dev/null || true

      $BB mkdir -p "$out/bin"
      $BB cp diff "$out/bin/diff"
      $BB chmod 755 "$out/bin/diff"
      [ -f cmp ] && $BB cp cmp "$out/bin/cmp" && $BB chmod 755 "$out/bin/cmp"

      test -x "$out/bin/diff"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, diffutils_src],
} | crunch.Derivation
