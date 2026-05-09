# Build flex 2.6.4 linked against musl using tcc-musl-v2.
#
# Refreshed flex with full feature set for later bison/binutils builds.
# Adapted from live-bootstrap steps/flex-2.6.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let flex_old = import "flex-2.5.11-musl.ncl" in

let flex_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/flex-2.6.4/sources
  # first-consumer: bootstrap/flex-2.6.4-musl.ncl
  url = "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz",
  hash = "sha256-HBPAUDpqF4HYocMD3YDIk6OaIUrZw8Gk3K59OmHB6xU=",
  name = "flex-2.6.4-src",
} in

{
  name = "flex-2.6.4-musl",
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
      FLEX_OLD=$(find_input flex-2.5.11-musl)
      SRC=$(find_input flex-2.6.4-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX_OLD/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/flex264-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" flex-2.6.4
      $BB chmod -R u+w flex-2.6.4
      cd flex-2.6.4

      # Generate config.h
      $BB cat > config.h << 'CONFIG'
#define PACKAGE "flex"
#define PACKAGE_NAME "flex"
#define VERSION "2.6.4"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define STDC_HEADERS 1
#define M4 "/bin/m4"
CONFIG

      CFLAGS="-I. -Isrc -I$MUSL/include -DHAVE_CONFIG_H"

      # Compile flex sources from src/.  Required sources must not be
      # hidden behind partial object/link fallback: a missing object means this
      # bootstrap part is not proven.
      OBJS=""
      for src in buf ccl dfa ecs filter gen main misc nfa options parse \
                 regex scan scanflags scanopt skel sym tables tblcmp yylex; do
        obj="src/$src.o"
        tcc -c $CFLAGS "src/$src.c" -o "$obj" || { echo "ERROR: failed compiling src/$src.c" >&2; exit 1; }
        test -f "$obj" || { echo "ERROR: missing object $obj" >&2; exit 1; }
        OBJS="$OBJS $obj"
      done
      test -n "$OBJS" || { echo "ERROR: no flex objects were built" >&2; exit 1; }

      if ! tcc -static $OBJS -o flex -lm; then
        tcc -static $OBJS -o flex || { echo "ERROR: failed linking flex" >&2; exit 1; }
      fi
      test -x flex || { echo "ERROR: linked flex missing" >&2; exit 1; }

      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
      $BB cp flex "$out/bin/flex"
      $BB chmod 755 "$out/bin/flex"
      cd "$out/bin"
      $BB ln -s flex lex
      $BB ln -s flex flex++

      # Install libfl if source has it; otherwise create the required yywrap
      # archive explicitly and fail if the archive is absent.
      cd "$WORK/flex-2.6.4"
      if [ -f src/libfl.a ]; then
        $BB cp src/libfl.a "$out/lib/libfl.a"
      else
        $BB cat > /tmp/libfl_main.c << 'LIBFL'
int yywrap(void) { return 1; }
LIBFL
        tcc -c -I"$MUSL/include" /tmp/libfl_main.c -o /tmp/libfl_main.o || { echo "ERROR: failed compiling libfl yywrap" >&2; exit 1; }
        tcc -ar cr "$out/lib/libfl.a" /tmp/libfl_main.o || { echo "ERROR: failed archiving libfl.a" >&2; exit 1; }
      fi
      test -f "$out/lib/libfl.a" || { echo "ERROR: installed libfl.a missing" >&2; exit 1; }

      # Install FlexLexer.h if present and make the conditional boundary explicit.
      if [ -f src/FlexLexer.h ]; then
        $BB cp src/FlexLexer.h "$out/include/FlexLexer.h"
        test -f "$out/include/FlexLexer.h" || { echo "ERROR: installed FlexLexer.h missing" >&2; exit 1; }
      fi

      test -x "$out/bin/flex" || { echo "ERROR: installed flex missing" >&2; exit 1; }
      test -L "$out/bin/lex" || { echo "ERROR: installed lex symlink missing" >&2; exit 1; }
      test -L "$out/bin/flex++" || { echo "ERROR: installed flex++ symlink missing" >&2; exit 1; }
      "$out/bin/flex" --version >/tmp/flex-version.txt || { echo "ERROR: flex --version smoke failed" >&2; exit 1; }
      $BB grep -q 'flex 2.6.4' /tmp/flex-version.txt || { echo "ERROR: flex version output mismatch" >&2; exit 1; }
      $BB cat > /tmp/flex-smoke.l << 'SMOKE'
%%
hello  ECHO;
%%
int yywrap(void) { return 1; }
SMOKE
      "$out/bin/flex" -o /tmp/flex-smoke.c /tmp/flex-smoke.l || { echo "ERROR: flex scanner generation smoke failed" >&2; exit 1; }
      test -f /tmp/flex-smoke.c || { echo "ERROR: flex smoke output missing" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4, flex_old, flex_src],
} | crunch.Derivation
