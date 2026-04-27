# Build gawk 3.0.4 linked against musl using tcc-musl-v2.
#
# awk is needed by autoconf/automake for generated source processing.
# Adapted from live-bootstrap steps/gawk-3.0.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let gawk_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/gawk/gawk-3.0.4.tar.gz",
  hash = "sha256-Vvj7dAxXMJhOHjWRd6ikD4eyjIERORHR9UcWfBDjAkY=",
  name = "gawk-3.0.4-src",
} in

{
  name = "gawk-3.0.4-musl",
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
      SRC=$(find_input gawk-3.0.4-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/gawk-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" gawk-3.0.4
      $BB chmod -R u+w gawk-3.0.4
      cd gawk-3.0.4

      : > config.h

      CFLAGS="-I. -I$MUSL/include -DSTDC_HEADERS -DHAVE_STRING_H -DHAVE_UNISTD_H -DHAVE_STDLIB_H -DHAVE_FCNTL_H -DHAVE_LIMITS_H -DHAVE_LOCALE_H -DHAVE_SYS_TYPES_H -DVERSION=\\\"3.0.4\\\" -DPACKAGE=\\\"gawk\\\" -DDEFPATH=\\\".:/usr/share/awk\\\""

      for src in array builtin eval field getopt getopt1 io main msg \
                 node re regex version; do
        tcc -c $CFLAGS $src.c -o $src.o 2>/dev/null || true
      done

      # Use pre-generated parser if available
      if [ -f awkgram.c ]; then
        tcc -c $CFLAGS awkgram.c -o awkgram.o
      elif [ -f awktab.c ]; then
        tcc -c $CFLAGS awktab.c -o awktab.o
      fi

      OBJS=""
      for f in *.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -o gawk -lm 2>/dev/null || tcc -static $OBJS -o gawk

      echo 'test' | ./gawk '{print "awk-ok"}'

      $BB mkdir -p "$out/bin"
      $BB cp gawk "$out/bin/gawk"
      $BB chmod 755 "$out/bin/gawk"
      cd "$out/bin"
      $BB ln -s gawk awk

      test -x "$out/bin/gawk"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, gawk_src],
} | crunch.Derivation
