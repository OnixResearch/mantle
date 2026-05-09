# Rebuild coreutils 5.0 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/coreutils-5.0 (musl rebuild).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let coreutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/coreutils-5.0/sources (musl rebuild)
  # first-consumer: bootstrap/coreutils-5.0-musl.ncl
  url = "https://mirrors.kernel.org/gnu/coreutils/coreutils-5.0.tar.bz2",
  hash = "sha256-VtfMrIYHZZ4YT21zYxctKfFEvGXDKerxOvCO7mmOPlA=",
  name = "coreutils-5.0-src",
} in

{
  name = "coreutils-5.0-musl",
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
      SRC=$(find_input coreutils-5.0-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/coreutils-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" coreutils-5.0
      $BB chmod -R u+w coreutils-5.0
      cd coreutils-5.0

      $BB cat > config.h << 'CONFIG'
#define PACKAGE "coreutils"
#define VERSION "5.0"
#define HAVE_FCNTL_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_LOCALE_H 1
#define HAVE_DECL_FREE 1
#define HAVE_DECL_MALLOC 1
#define HAVE_DECL_MEMRCHR 0
#define HAVE_DECL_STPCPY 0
#define HAVE_DECL_STRNDUP 0
#define HAVE_DECL_STRNLEN 0
#define HAVE_DECL_STRTOUL 1
#define HAVE_DECL_STRTOULL 1
#define HAVE_DECL_STRTOIMAX 0
#define HAVE_DECL_STRTOUMAX 0
#define HAVE_LCHOWN 1
#define HAVE_MKFIFO 1
#define HAVE_FTRUNCATE 1
#define STDC_HEADERS 1
#define _GNU_SOURCE 1
CONFIG

      CFLAGS="-I. -Ilib -Isrc -I$MUSL/include -DHAVE_CONFIG_H -D_FILE_OFFSET_BITS=64"

      LIBSRCS="lib/error lib/xmalloc lib/xstrdup lib/getopt lib/getopt1 lib/closeout lib/quotearg lib/quote lib/exitfail lib/hard-locale lib/xstrtol lib/xstrtoul lib/human lib/idcache lib/filemode lib/basename lib/stripslash lib/full-write lib/safe-read lib/safe-write lib/version-etc lib/long-options"
      OBJS=""
      for src in $LIBSRCS; do
        tcc -c $CFLAGS $src.c -o $src.o
        test -f "$src.o" || { echo "ERROR: missing library object $src.o" >&2; exit 1; }
        OBJS="$OBJS $src.o"
      done
      test -n "$OBJS" || { echo "ERROR: no coreutils library objects compiled" >&2; exit 1; }
      tcc -ar cr libcu.a $OBJS
      test -f libcu.a || { echo "ERROR: libcu.a not built" >&2; exit 1; }

      $BB mkdir -p "$out/bin"

      for util in cat chmod cp echo install ln ls mkdir mv rm rmdir                   sort test true false head tail wc basename dirname                   tr uniq expr tee touch; do
        tcc -static $CFLAGS "src/$util.c" -L. -lcu -o "$out/bin/$util"
        test -x "$out/bin/$util" || { echo "ERROR: missing installed utility $util" >&2; exit 1; }
      done

      cd "$out/bin"
      $BB ln -s test "["
      cd "$WORK/coreutils-5.0"

      for util in cat chmod cp echo install ln ls mkdir mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch "["; do
        test -x "$out/bin/$util" || { echo "ERROR: output contract missing $util" >&2; exit 1; }
      done

      "$out/bin/mkdir" /tmp/coreutils-smoke-dir
      "$out/bin/echo" smoke-ok > /tmp/coreutils-smoke-dir/input
      "$out/bin/cp" /tmp/coreutils-smoke-dir/input /tmp/coreutils-smoke-dir/output
      "$out/bin/cat" /tmp/coreutils-smoke-dir/output | $BB grep -q smoke-ok
      "$out/bin/test" -f /tmp/coreutils-smoke-dir/output
      "$out/bin/[" -f /tmp/coreutils-smoke-dir/output "]"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, coreutils_src],
} | crunch.Derivation
