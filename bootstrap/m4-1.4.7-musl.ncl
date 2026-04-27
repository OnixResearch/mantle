# Build m4 1.4.7 linked against musl using tcc-musl-v2.
#
# m4 is a macro processor required by autoconf. This version is
# built before bison/flex to keep the chain linear.
#
# Adapted from live-bootstrap steps/m4-1.4.7.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let m4_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/m4/m4-1.4.7.tar.bz2",
  hash = "sha256-qI892qfInPTDQoQ4W+QcqF6RNTacMz/fojLzv0giMhM=",
  name = "m4-1.4.7-src",
} in

{
  name = "m4-1.4.7-musl",
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
      SRC=$(find_input m4-1.4.7-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/m4-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" m4-1.4.7
      $BB chmod -R u+w m4-1.4.7
      cd m4-1.4.7

      : > config.h

      CFLAGS="-I. -Isrc -I$MUSL/include -DHAVE_CONFIG_H -DHAVE_STDLIB_H -DHAVE_STRING_H -DHAVE_UNISTD_H -DHAVE_ERRNO_H -DHAVE_SYS_TYPES_H -DHAVE_LIMITS_H -DHAVE_LOCALE_H -DHAVE_SIGNAL_H -DHAVE_FCNTL_H -DHAVE_DIRENT_H -DVERSION=\\\"1.4.7\\\" -DPACKAGE=\\\"m4\\\""

      # Compile m4 sources
      for src in m4 builtin debug eval format freeze input macro \
                 output path stackovf symtab; do
        tcc -c $CFLAGS src/$src.c -o src/$src.o
      done

      # Compile lib support
      for src in obstack regex getopt getopt1 xmalloc xstrdup error; do
        tcc -c $CFLAGS lib/$src.c -o lib/$src.o 2>/dev/null || true
      done

      OBJS=""
      for f in src/*.o lib/*.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -o m4

      echo 'define(FOO,bar)FOO' | ./m4

      $BB mkdir -p "$out/bin"
      $BB cp m4 "$out/bin/m4"
      $BB chmod 755 "$out/bin/m4"

      test -x "$out/bin/m4"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4_src],
} | crunch.Derivation
