# Build bison 2.3 linked against musl using tcc-musl-v2.
#
# Legacy bison needed by some older autotools-era configure scripts.
# Adapted from live-bootstrap steps/bison-2.3.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let flex = import "flex-2.6.4-musl.ncl" in

let bison_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/bison/bison-2.3.tar.bz2",
  hash = "sha256-A7EYDEuY+Hw8DHostT2RBv+kjkMumqDlshfpCi2I77U=",
  name = "bison-2.3-src",
} in

{
  name = "bison-2.3-musl",
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
      SRC=$(find_input bison-2.3-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$FLEX/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bison23-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bison-2.3
      $BB chmod -R u+w bison-2.3
      cd bison-2.3

      # Generate config.h
      $BB cat > config.h << 'CONFIG'
#define PACKAGE "bison"
#define PACKAGE_NAME "bison"
#define VERSION "2.3"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_SYS_TYPES_H 1
#define STDC_HEADERS 1
#define M4 "m4"
#define M4_GNU_OPTION ""
#define LOCALEDIR "/tmp/locale"
CONFIG

      CFLAGS="-I. -Ilib -I$MUSL/include -DHAVE_CONFIG_H"

      # Compile lib support
      for src in argmatch basename bitset bitsetv error exitfail getopt getopt1 \
                 hash mbswidth quotearg subpipe xalloc-die xmalloc xstrndup; do
        tcc -c $CFLAGS lib/$src.c -o lib/$src.o 2>/dev/null || true
      done
      LIBOBJS=""
      for f in lib/*.o; do [ -f "$f" ] && LIBOBJS="$LIBOBJS $f"; done
      tcc -ar cr libbison.a $LIBOBJS 2>/dev/null || true

      # Compile bison sources
      for src in LR0 assoc closure complain conflicts derives \
                 files getargs gram lalr location main muscle_tab \
                 nullable output parse-gram print print_graph reader \
                 reduce relation scan-gram scan-skel state symlist \
                 symtab tables uniqstr; do
        tcc -c $CFLAGS src/$src.c -o src/$src.o 2>/dev/null || true
      done

      OBJS=""
      for f in src/*.o; do [ -f "$f" ] && OBJS="$OBJS $f"; done
      tcc -static $OBJS -L. -lbison -o bison 2>/dev/null || tcc -static $OBJS -o bison

      $BB mkdir -p "$out/bin" "$out/share/bison"
      $BB cp bison "$out/bin/bison"
      $BB chmod 755 "$out/bin/bison"

      # Copy data files needed by bison at runtime
      if [ -d data ]; then
        $BB cp -r data/* "$out/share/bison/" 2>/dev/null || true
      fi

      test -x "$out/bin/bison"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4, flex, bison_src],
} | crunch.Derivation
