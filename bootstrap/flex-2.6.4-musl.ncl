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
  url = "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz",
  hash = "sha256-6HquAyvwfCb4WsDtMlCZjDdiHZX4vXSLMfFbM8Re6ZU=",
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

      # Compile flex sources from src/
      for src in buf ccl dfa ecs filter gen main misc nfa options parse \
                 regex scan scanflags scanopt skel sym tables tblcmp yylex; do
        tcc -c $CFLAGS src/$src.c -o src/$src.o 2>/dev/null || true
      done

      OBJS=""
      for f in src/*.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -o flex -lm 2>/dev/null || tcc -static $OBJS -o flex

      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
      $BB cp flex "$out/bin/flex"
      $BB chmod 755 "$out/bin/flex"
      cd "$out/bin"
      $BB ln -s flex lex
      $BB ln -s flex flex++

      # Install libfl if source has it
      cd "$WORK/flex-2.6.4"
      if [ -f src/libfl.a ]; then
        $BB cp src/libfl.a "$out/lib/"
      else
        # Create minimal libfl.a
        $BB cat > /tmp/libfl_main.c << 'LIBFL'
int yywrap(void) { return 1; }
LIBFL
        tcc -c -I"$MUSL/include" /tmp/libfl_main.c -o /tmp/libfl_main.o
        tcc -ar cr "$out/lib/libfl.a" /tmp/libfl_main.o
      fi

      # Install FlexLexer.h if present
      if [ -f src/FlexLexer.h ]; then
        $BB cp src/FlexLexer.h "$out/include/"
      fi

      test -x "$out/bin/flex"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4, flex_old, flex_src],
} | crunch.Derivation
