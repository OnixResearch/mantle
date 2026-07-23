# Build coreutils 5.0 with the source-built TinyCC stage.
#
# Adapted from live-bootstrap steps/coreutils-5.0 (tcc build).
# This provides standard POSIX utilities needed by autotools.
#
# Source: https://github.com/fosslinux/live-bootstrap
# Licensed under GPL-3.0-or-later.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let mes = crunch.derivationFile "mes.ncl" in
let tcc = crunch.derivationFile "tinycc.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-tcc.ncl" in
let tar = crunch.derivationFile "tar-tcc.ncl" in
let bzip2 = crunch.derivationFile "bzip2-tcc.ncl" in

let coreutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/coreutils-5.0/sources (tcc build)
  # first-consumer: bootstrap/coreutils-5.0-tcc.ncl
  url = "https://mirrors.kernel.org/gnu/coreutils/coreutils-5.0.tar.bz2",
  hash = "sha256-VtfMrIYHZZ4YT21zYxctKfFEvGXDKerxOvCO7mmOPlA=",
  name = "coreutils-5.0-src",
} in

{
  name = "coreutils-5.0-tcc",
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

      TCC=$(find_input tinycc-0.9.27)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      SED=$(find_input sed-4.0.9-tcc)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-tcc)
      SRC=$(find_input coreutils-5.0-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$SED/bin:$TAR/bin:$BZIP2/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/coreutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" coreutils-5.0
      $BB chmod -R u+w coreutils-5.0
      cd coreutils-5.0

      # The live-bootstrap source carries generated parser/headers.  Compile the
      # bounded utility set directly so this stage does not depend on host
      # configure, Automake, or shell wrappers.
      $BB cat > config.h << 'CONFIG'
#define PACKAGE "coreutils"
#define PACKAGE_NAME "coreutils"
#define VERSION "5.0"
#define HAVE_FCNTL_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_INTTYPES_H 1
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

      CFLAGS="-I. -Ilib -Isrc -DHAVE_CONFIG_H -D_FILE_OFFSET_BITS=64"
      LIBSRCS="lib/error lib/xmalloc lib/xstrdup lib/getopt lib/getopt1 lib/closeout lib/quotearg lib/quote lib/exitfail lib/hard-locale lib/xstrtol lib/xstrtoul lib/human lib/idcache lib/filemode lib/basename lib/stripslash lib/full-write lib/safe-read lib/safe-write lib/version-etc lib/long-options"
      OBJS=""
      for src in $LIBSRCS; do
        echo "coreutils-5.0-library-source=$src.c" >&2
        tcc -c $CFLAGS "$src.c" -o "$src.o"
        test -f "$src.o" || { echo "ERROR: missing library object $src.o" >&2; exit 1; }
        OBJS="$OBJS $src.o"
      done
      test -n "$OBJS" || { echo "ERROR: no coreutils library objects compiled" >&2; exit 1; }
      tcc -ar cr libcu.a $OBJS
      test -f libcu.a || { echo "ERROR: libcu.a not built" >&2; exit 1; }

      $BB mkdir -p "$out/bin"
      build_util() {
        name=$1
        shift
        tcc -static $CFLAGS "$@" -L. -lcu -o "$out/bin/$name"
        test -f "$out/bin/$name" || { echo "ERROR: $name not linked" >&2; exit 1; }
        $BB chmod 755 "$out/bin/$name"
      }

      build_util cat src/cat.c
      build_util chmod src/chmod.c
      build_util cp src/cp.c src/copy.c src/cp-hash.c
      build_util echo src/echo.c
      build_util install src/install.c src/copy.c src/cp-hash.c
      build_util ln src/ln.c
      build_util ls src/ls.c src/ls-ls.c
      build_util mkdir src/mkdir.c
      build_util mv src/mv.c src/copy.c src/cp-hash.c src/remove.c
      build_util rm src/rm.c src/remove.c
      build_util rmdir src/rmdir.c
      build_util sort src/sort.c
      build_util test src/test.c
      build_util true src/true.c
      build_util false src/false.c
      build_util head src/head.c
      build_util tail src/tail.c
      build_util wc src/wc.c
      build_util basename src/basename.c
      build_util dirname src/dirname.c
      build_util tr src/tr.c
      build_util uniq src/uniq.c
      build_util expr src/expr.c
      build_util tee src/tee.c
      build_util touch src/touch.c

      $BB cp "$out/bin/test" "$out/bin/["
      $BB chmod 755 "$out/bin/["

      for util in cat chmod cp echo install ln ls mkdir mv rm rmdir sort test true false head tail wc basename dirname tr uniq expr tee touch "["; do
        test -x "$out/bin/$util" || { echo "ERROR: output contract missing $util" >&2; exit 1; }
        if $BB head -c 2 "$out/bin/$util" | $BB grep -q '^#!'; then
          echo "ERROR: source-built coreutils output is a script: $util" >&2
          exit 1
        fi
      done

      "$out/bin/mkdir" /tmp/coreutils-smoke-dir
      "$out/bin/echo" smoke-ok > /tmp/coreutils-smoke-dir/input
      "$out/bin/cp" /tmp/coreutils-smoke-dir/input /tmp/coreutils-smoke-dir/output
      "$out/bin/cat" /tmp/coreutils-smoke-dir/output | $BB grep -q smoke-ok
      "$out/bin/test" -f /tmp/coreutils-smoke-dir/output
      "$out/bin/[" -f /tmp/coreutils-smoke-dir/output "]"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, sed, tar, bzip2, coreutils_src],
} | crunch.Derivation
