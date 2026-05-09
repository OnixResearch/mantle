# Build coreutils 6.10 linked against musl using tcc-musl-v2.
#
# Provides date, mktemp, sha256sum, and expanded utility set needed
# by autotools and Perl configure scripts.
#
# Adapted from live-bootstrap steps/coreutils-6.10.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let coreutils_old = import "coreutils-5.0-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bzip2 = import "bzip2-1.0.8-musl.ncl" in

let coreutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/coreutils-6.10/sources
  # first-consumer: bootstrap/coreutils-6.10-musl.ncl
  url = "https://mirrors.kernel.org/gnu/coreutils/coreutils-6.10.tar.gz",
  hash = "sha256-WZ798TmMBEj5l+W0liycJjR4LmB0eBNy3YjsxB55Y+I=",
  name = "coreutils-6.10-src",
} in

{
  name = "coreutils-6.10-musl",
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
      CU_OLD=$(find_input coreutils-5.0-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input coreutils-6.10-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$CU_OLD/bin:$DIFF/bin:$BZIP2/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/coreutils610-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" coreutils-6.10
      $BB chmod -R u+w coreutils-6.10
      cd coreutils-6.10

      $BB cat > config.h << 'CONFIG'
#define PACKAGE "coreutils"
#define PACKAGE_NAME "coreutils"
#define VERSION "6.10"
#define HAVE_FCNTL_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_TIME_H 1
#define HAVE_SYS_WAIT_H 1
#define STDC_HEADERS 1
#define _GNU_SOURCE 1
#define HAVE_DECL_FREE 1
#define HAVE_DECL_MALLOC 1
#define HAVE_DECL_STRTOUL 1
#define HAVE_DECL_STRTOULL 1
#define HAVE_LSTAT 1
#define HAVE_LCHOWN 1
#define HAVE_FTRUNCATE 1
CONFIG

      CFLAGS="-I. -Ilib -Isrc -I$MUSL/include -DHAVE_CONFIG_H -D_FILE_OFFSET_BITS=64"

      # Build lib
      LIBOBJS=""
      for src in lib/*.c; do
        base=$($BB basename "$src" .c)
        tcc -c $CFLAGS "$src" -o "lib/$base.o"
        test -f "lib/$base.o" || { echo "ERROR: missing library object lib/$base.o" >&2; exit 1; }
        LIBOBJS="$LIBOBJS lib/$base.o"
      done
      test -n "$LIBOBJS" || { echo "ERROR: no coreutils library objects compiled" >&2; exit 1; }
      tcc -ar cr libcu.a $LIBOBJS
      test -f libcu.a || { echo "ERROR: libcu.a not built" >&2; exit 1; }

      $BB mkdir -p "$out/bin"

      for util in cat chmod cp date echo install ln ls mkdir mktemp mv                   rm rmdir sort test true false head tail wc basename                   dirname tr uniq expr tee touch sha256sum; do
        tcc -static $CFLAGS "src/$util.c" -L. -lcu -o "$out/bin/$util"
        test -x "$out/bin/$util" || { echo "ERROR: missing installed utility $util" >&2; exit 1; }
      done

      cd "$out/bin"
      $BB ln -s test "["
      cd "$WORK/coreutils-6.10"

      for util in cat chmod cp date echo install ln ls mkdir mktemp mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch sha256sum "["; do
        test -x "$out/bin/$util" || { echo "ERROR: output contract missing $util" >&2; exit 1; }
      done
      "$out/bin/mkdir" /tmp/coreutils610-smoke
      "$out/bin/echo" smoke-ok > /tmp/coreutils610-smoke/input
      "$out/bin/cp" /tmp/coreutils610-smoke/input /tmp/coreutils610-smoke/output
      "$out/bin/cat" /tmp/coreutils610-smoke/output | $BB grep -q smoke-ok
      "$out/bin/mktemp" /tmp/coreutils610-smoke/tmp.XXXXXX >/tmp/coreutils610-smoke/mktemp-path
      "$out/bin/sha256sum" /tmp/coreutils610-smoke/output >/tmp/coreutils610-smoke/sha256
      "$out/bin/test" -f /tmp/coreutils610-smoke/output
      "$out/bin/[" -f /tmp/coreutils610-smoke/output "]"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, coreutils_old, diffutils, bzip2, coreutils_src],
} | crunch.Derivation
