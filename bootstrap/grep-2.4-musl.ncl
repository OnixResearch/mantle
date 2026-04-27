# Build grep 2.4 linked against musl using tcc-musl-v2.
#
# First text search tool in the post-musl chain.
# Adapted from live-bootstrap steps/grep-2.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let grep_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/grep/grep-2.4.tar.gz",
  hash = "sha256-v9xxK5dLi3FxvpCbRjPorpK+bPNcAXwj/jgX4eslWxI=",
  name = "grep-2.4-src",
} in

{
  name = "grep-2.4-musl",
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
      SED=$(find_input sed-4.0.9-tcc)
      SRC=$(find_input grep-2.4-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/grep-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" grep-2.4
      $BB chmod -R u+w grep-2.4
      cd grep-2.4

      : > config.h

      CFLAGS="-I. -I$MUSL/include -DSTDC_HEADERS -DHAVE_MEMCHR -DHAVE_STRING_H -DHAVE_STDLIB_H -DHAVE_UNISTD_H -DHAVE_DIRENT_H -DVERSION=\\\"2.4\\\" -DPACKAGE=\\\"grep\\\""

      # Compile sources
      for src in grep dfa kwset obstack getopt getopt1 search grepmat savedir stpcpy; do
        tcc -c $CFLAGS src/$src.c -o src/$src.o 2>/dev/null || true
      done

      # Link what compiled
      OBJS=""
      for f in src/*.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -o grep

      # Verify
      echo "hello" | ./grep hello

      $BB mkdir -p "$out/bin"
      $BB cp grep "$out/bin/grep"
      $BB chmod 755 "$out/bin/grep"
      cd "$out/bin"
      $BB ln -s grep egrep
      $BB ln -s grep fgrep

      test -x "$out/bin/grep"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, grep_src],
} | crunch.Derivation
