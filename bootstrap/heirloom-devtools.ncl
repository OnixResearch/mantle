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
  # provenance: fosslinux/live-bootstrap steps/heirloom-devtools-070527/sources
  # first-consumer: bootstrap/heirloom-devtools.ncl
  url = "http://downloads.sourceforge.net/project/heirloom/heirloom-devtools/070527/heirloom-devtools-070527.tar.bz2",
  hash = "sha256-yJ2pXfdovrJ9VwR1kq0Op5J8bRq8zXMohk8GCRfX7ZY=",
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

      # Patch Makefile to use tcc. These edits are part of the declared build
      # contract, so missing or incompatible `mk.config` must fail closed.
      test -f mk.config || { echo "ERROR: heirloom mk.config missing" >&2; exit 1; }
      sed -i 's|CC = .*|CC = tcc|' mk.config
      sed -i 's|CFLAGS = .*|CFLAGS = -I'"$MUSL"'/include|' mk.config
      sed -i 's|LDFLAGS = .*|LDFLAGS = -static|' mk.config
      sed -i 's|AR = .*|AR = tcc -ar|' mk.config
      sed -i 's|RANLIB = .*|RANLIB = true|' mk.config
      sed -i 's|INSTALL = .*|INSTALL = /bin/busybox cp|' mk.config

      # Set prefix
      sed -i 's|PREFIX = .*|PREFIX = '"$out"'|' mk.config

      build_manual() {
        tool="$1"
        shift
        cd "$WORK/heirloom-devtools-070527/$tool"
        OBJS=""
        for src in "$@"; do
          obj="$src.o"
          tcc -c -I"$MUSL/include" "$src.c" -o "$obj"
          test -f "$obj" || { echo "ERROR: missing $tool object $obj" >&2; exit 1; }
          OBJS="$OBJS $obj"
        done
        test -n "$OBJS" || { echo "ERROR: no $tool objects compiled" >&2; exit 1; }
        tcc -static $OBJS -o "$tool"
        test -x "$tool" || { echo "ERROR: $tool not linked" >&2; exit 1; }
      }

      # Build yacc and lex. The make path is preferred, but the direct compile
      # path is still explicit and fail-closed rather than best-effort.
      cd yacc
      make CC=tcc AR="tcc -ar" CFLAGS="-I$MUSL/include" LDFLAGS="-static" || build_manual yacc y1 y2 y3 y4
      test -x "$WORK/heirloom-devtools-070527/yacc/yacc" || { echo "ERROR: yacc build output missing" >&2; exit 1; }

      cd "$WORK/heirloom-devtools-070527/lex"
      make CC=tcc AR="tcc -ar" CFLAGS="-I$MUSL/include" LDFLAGS="-static" || build_manual lex main sub header search nfa
      test -x "$WORK/heirloom-devtools-070527/lex/lex" || { echo "ERROR: lex build output missing" >&2; exit 1; }

      $BB mkdir -p "$out/bin"
      for tool in yacc lex; do
        f="$WORK/heirloom-devtools-070527/$tool/$tool"
        test -x "$f" || { echo "ERROR: $tool binary missing before install" >&2; exit 1; }
        $BB cp "$f" "$out/bin/$tool"
        $BB chmod 755 "$out/bin/$tool"
      done

      test -x "$out/bin/yacc" || { echo "ERROR: installed yacc missing" >&2; exit 1; }
      test -x "$out/bin/lex" || { echo "ERROR: installed lex missing" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, heirloom_src],
} | crunch.Derivation
