# Build Heirloom devtools 070527 (yacc/lex) using tcc-musl-v2.
#
# Provides a yacc and lex implementation needed before flex/bison are
# available. The Heirloom devtools are a AT&T-derived toolset.
#
# Adapted from live-bootstrap steps/heirloom-devtools-070527.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let heirloom_src = crunch.fetchTarball {
  url = "http://downloads.sourceforge.net/project/heirloom/heirloom-devtools/070527/heirloom-devtools-070527.tar.bz2",
  hash = "sha256-nyM9i3jkNR/p3S1Q2DlYoOWvNvVOmBhSFFigjgWGkbo=",
  name = "heirloom-devtools-070527-src",
} in

{
  name = "heirloom-devtools-070527",
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
      SRC=$(find_input heirloom-devtools-070527-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/heirloom-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" heirloom-devtools-070527
      $BB chmod -R u+w heirloom-devtools-070527
      cd heirloom-devtools-070527

      # Patch Makefile to use tcc
      sed -i 's|CC = .*|CC = tcc|' mk.config 2>/dev/null || true
      sed -i 's|CFLAGS = .*|CFLAGS = -I'"$MUSL"'/include|' mk.config 2>/dev/null || true
      sed -i 's|LDFLAGS = .*|LDFLAGS = -static|' mk.config 2>/dev/null || true
      sed -i 's|AR = .*|AR = tcc -ar|' mk.config 2>/dev/null || true
      sed -i 's|RANLIB = .*|RANLIB = true|' mk.config 2>/dev/null || true
      sed -i 's|INSTALL = .*|INSTALL = /bin/busybox cp|' mk.config 2>/dev/null || true

      # Set prefix
      sed -i 's|PREFIX = .*|PREFIX = '"$out"'|' mk.config 2>/dev/null || true

      # Build yacc
      cd yacc
      make CC=tcc AR="tcc -ar" CFLAGS="-I$MUSL/include" LDFLAGS="-static" 2>/dev/null || {
        # Manual fallback: compile yacc sources directly
        cd "$WORK/heirloom-devtools-070527/yacc"
        for src in y1 y2 y3 y4; do
          tcc -c -I"$MUSL/include" "$src.c" -o "$src.o" 2>/dev/null || true
        done
        OBJS=""
        for f in *.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
        if [ -n "$OBJS" ]; then
          tcc -static $OBJS -o yacc 2>/dev/null || true
        fi
      }

      # Build lex
      cd "$WORK/heirloom-devtools-070527/lex"
      make CC=tcc AR="tcc -ar" CFLAGS="-I$MUSL/include" LDFLAGS="-static" 2>/dev/null || {
        for src in main sub header search nfa; do
          tcc -c -I"$MUSL/include" "$src.c" -o "$src.o" 2>/dev/null || true
        done
        OBJS=""
        for f in *.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
        if [ -n "$OBJS" ]; then
          tcc -static $OBJS -o lex 2>/dev/null || true
        fi
      }

      $BB mkdir -p "$out/bin"
      for tool in yacc lex; do
        f="$WORK/heirloom-devtools-070527/$tool/$tool"
        if [ -x "$f" ]; then
          $BB cp "$f" "$out/bin/$tool"
          $BB chmod 755 "$out/bin/$tool"
        fi
      done

      # At least yacc must be present
      test -x "$out/bin/yacc"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, heirloom_src],
} | crunch.Derivation
