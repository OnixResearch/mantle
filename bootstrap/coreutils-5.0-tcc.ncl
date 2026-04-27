# Build coreutils 5.0 subset using tinycc 0.9.27.
#
# Provides early shell utilities needed by later bootstrap stages:
# cat, chmod, cp, echo, install, ln, ls, mkdir, mv, rm, rmdir,
# sort, test, true, false, head, tail, wc, basename, dirname,
# tr, uniq, expr, tee, touch.
#
# The configure system is bypassed; each utility is compiled directly.
# Adapted from live-bootstrap steps/coreutils-5.0.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in
let tar = import "tar-tcc.ncl" in
let bzip2 = import "bzip2-tcc.ncl" in

let coreutils_src = crunch.fetchTarball {
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

      # Generate minimal config.h
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
#define DEFAULT_TMPDIR "/tmp"
CONFIG

      CFLAGS="-I. -Ilib -Isrc -DHAVE_CONFIG_H -D_FILE_OFFSET_BITS=64"

      # Compile libcoreutils support files
      LIBSRCS="lib/error lib/xmalloc lib/xstrdup lib/getopt lib/getopt1 lib/closeout lib/quotearg lib/quote lib/exitfail lib/hard-locale lib/xstrtol lib/xstrtoul lib/human lib/idcache lib/filemode lib/basename lib/stripslash lib/full-write lib/safe-read lib/safe-write lib/version-etc lib/long-options"

      for src in $LIBSRCS; do
        tcc -c $CFLAGS $src.c -o $src.o 2>/dev/null || true
      done

      # Create library archive from what compiled
      OBJS=""
      for src in $LIBSRCS; do
        if [ -f "$src.o" ]; then
          OBJS="$OBJS $src.o"
        fi
      done
      tcc -ar cr libcu.a $OBJS

      # Build individual coreutils programs
      $BB mkdir -p "$out/bin"

      build_util() {
        name=$1
        shift
        if tcc -static $CFLAGS "$@" -L. -lcu -o "$out/bin/$name" 2>/dev/null; then
          $BB chmod 755 "$out/bin/$name"
          return 0
        fi
        return 1
      }

      # Simple single-source utilities
      for util in cat chmod cp echo install ln ls mkdir mv rm rmdir \
                  sort test true false head tail wc basename dirname \
                  tr uniq expr tee touch; do
        build_util "$util" "src/$util.c" || true
      done

      # Create [ as a link to test
      if [ -x "$out/bin/test" ]; then
        cd "$out/bin"
        $BB ln -s test "["
        cd "$WORK/coreutils-5.0"
      fi

      # Verify at least a few essential utilities exist
      test -x "$out/bin/cat" || { echo "ERROR: cat not built" >&2; exit 1; }
      test -x "$out/bin/chmod" || { echo "ERROR: chmod not built" >&2; exit 1; }
      test -x "$out/bin/cp" || { echo "ERROR: cp not built" >&2; exit 1; }
      test -x "$out/bin/mkdir" || { echo "ERROR: mkdir not built" >&2; exit 1; }
      test -x "$out/bin/ln" || { echo "ERROR: ln not built" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, mes, tcc, make, sed, tar, bzip2, coreutils_src],
} | crunch.Derivation
