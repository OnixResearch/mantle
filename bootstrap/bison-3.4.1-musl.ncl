# Build bison 3.4.1 linked against musl using tcc-musl-v2.
#
# Modern bison for binutils configure and later GCC builds.
# Adapted from live-bootstrap steps/bison-3.4.1.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let flex = import "flex-2.6.4-musl.ncl" in
let bison_old = import "bison-2.3-musl.ncl" in

let bison_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/bison-3.4.1/sources
  # first-consumer: bootstrap/bison-3.4.1-musl.ncl
  url = "https://mirrors.kernel.org/gnu/bison/bison-3.4.1.tar.xz",
  hash = "sha256-vKqyf0sdiIdtfaEILKmi1ULhs+AqVSboYSpGhgEsYsI=",
  name = "bison-3.4.1-src",
} in

{
  name = "bison-3.4.1-musl",
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
      FLEX=$(find_input flex-2.6.4-musl)
      BISON_OLD=$(find_input bison-2.3-musl)
      SRC=$(find_input bison-3.4.1-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$BISON_OLD/bin:$STAGE0/bin:$PATH"
      export BISON_PKGDATADIR="$BISON_OLD/share/bison"

      WORK=/tmp/bison341-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bison-3.4.1
      $BB chmod -R u+w bison-3.4.1
      cd bison-3.4.1

      $BB cat > config.h << 'CONFIG'
#define PACKAGE "bison"
#define PACKAGE_NAME "GNU Bison"
#define PACKAGE_VERSION "3.4.1"
#define VERSION "3.4.1"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_WAIT_H 1
#define HAVE_WCTYPE_H 1
#define HAVE_WCHAR_H 1
#define STDC_HEADERS 1
#define M4 "m4"
#define M4_GNU_OPTION ""
#define LOCALEDIR "/tmp/locale"
#define PKGDATADIR "/tmp/bison-data"
#define RENAME_OPEN_FILE_WORKS 1
CONFIG

      CFLAGS="-I. -Ilib -I$MUSL/include -DHAVE_CONFIG_H -DEXEEXT=\\\"\\\""

      # Compile lib support
      LIBOBJS=""
      for src in lib/*.c; do
        base=$($BB basename "$src" .c)
        tcc -c $CFLAGS "$src" -o "lib/$base.o"
        test -f "lib/$base.o" || { echo "ERROR: missing lib object lib/$base.o" >&2; exit 1; }
        LIBOBJS="$LIBOBJS lib/$base.o"
      done
      test -n "$LIBOBJS" || { echo "ERROR: no bison lib objects compiled" >&2; exit 1; }
      tcc -ar cr libbison.a $LIBOBJS
      test -f libbison.a || { echo "ERROR: libbison.a not built" >&2; exit 1; }

      # Compile bison sources
      OBJS=""
      for src in src/*.c; do
        base=$($BB basename "$src" .c)
        tcc -c $CFLAGS "$src" -o "src/$base.o"
        test -f "src/$base.o" || { echo "ERROR: missing source object src/$base.o" >&2; exit 1; }
        OBJS="$OBJS src/$base.o"
      done
      test -n "$OBJS" || { echo "ERROR: no bison source objects compiled" >&2; exit 1; }

      tcc -static $OBJS -L. -lbison -o bison
      test -f bison || { echo "ERROR: bison executable not linked" >&2; exit 1; }

      $BB mkdir -p "$out/bin" "$out/share/bison"
      $BB cp bison "$out/bin/bison"
      $BB chmod 755 "$out/bin/bison"

      test -d data || { echo "ERROR: bison data directory missing" >&2; exit 1; }
      $BB cp -r data/* "$out/share/bison/"

      test -x "$out/bin/bison" || { echo "ERROR: installed bison missing" >&2; exit 1; }
      test -d "$out/share/bison" || { echo "ERROR: installed bison data directory missing" >&2; exit 1; }
      test -f "$out/share/bison/yacc.c" || { echo "ERROR: installed yacc.c skeleton missing" >&2; exit 1; }
      "$out/bin/bison" --version >/tmp/bison341-version.txt
      $BB grep -q 'bison' /tmp/bison341-version.txt || { echo "ERROR: bison version smoke failed" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4, flex, bison_old, bison_src],
} | crunch.Derivation
