# Build flex 2.5.11 linked against musl using tcc-musl-v2.
#
# Initial flex provides lex functionality for bison builds.
# Adapted from live-bootstrap steps/flex-2.5.11.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let heirloom = import "heirloom-devtools.ncl" in

let flex_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/flex-2.5.11/sources
  # first-consumer: bootstrap/flex-2.5.11-musl.ncl
  url = "https://github.com/westes/flex/archive/d160f0247ba1611aa59d28f027d6292ba24abb50.tar.gz",
  hash = "sha256-FZW6ydHxZ2gUpxDcAfMkwNoYp32RbpG5T2y6MuSe86Y=",
  name = "flex-2.5.11-src",
} in

{
  name = "flex-2.5.11-musl",
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
      M4=$(find_input m4-1.4.7-musl)
      HEIRLOOM=$(find_input heirloom-devtools-070527)
      SRC=$(find_input flex-2.5.11-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$HEIRLOOM/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/flex-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" flex-2.5.11
      $BB chmod -R u+w flex-2.5.11
      cd flex-2.5.11

      : > config.h

      CFLAGS="-I. -I$MUSL/include -DHAVE_STRING_H -DHAVE_LIMITS_H -DHAVE_UNISTD_H -DSTDC_HEADERS -DVERSION=\\\"2.5.11\\\""

      # Compile flex sources (pre-generated scanner if available)
      OBJS=""
      for src in ccl dfa ecs gen main misc nfa parse scan skel sym tblcmp yylex; do
        tcc -c $CFLAGS "$src.c" -o "$src.o"
        test -f "$src.o" || { echo "ERROR: missing flex object $src.o" >&2; exit 1; }
        OBJS="$OBJS $src.o"
      done
      test -n "$OBJS" || { echo "ERROR: no flex objects compiled" >&2; exit 1; }
      tcc -static $OBJS -o flex
      test -f flex || { echo "ERROR: flex executable not linked" >&2; exit 1; }

      $BB mkdir -p "$out/bin"
      $BB cp flex "$out/bin/flex"
      $BB chmod 755 "$out/bin/flex"
      cd "$out/bin"
      $BB ln -s flex lex

      test -x "$out/bin/flex" || { echo "ERROR: installed flex missing" >&2; exit 1; }
      test -x "$out/bin/lex" || { echo "ERROR: installed lex missing" >&2; exit 1; }
      "$out/bin/flex" --version >/tmp/flex2511-version.txt
      $BB grep -q 'flex' /tmp/flex2511-version.txt || { echo "ERROR: flex version smoke failed" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4, heirloom, flex_src],
} | crunch.Derivation
