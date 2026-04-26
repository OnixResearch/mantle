# Build GNU tar 1.12 using tinycc 0.9.27 + make 3.82.
#
# tar replaces the stage0-posix untar utility with a full-featured
# implementation that handles more archive formats.
# Adapted from live-bootstrap steps/tar-1.12/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in
let gzip = import "gzip-tcc.ncl" in

let tar_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/tar/tar-1.12.tar.gz",
  hash = "sha256-xsN+iIsTbM76uQPFEUn0t71lnWnUrqISRfYQU6V6pgo=",
  name = "tar-1.12-src",
} in

{
  name = "tar-1.12-tcc",
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
      GZIP=$(find_input gzip-1.2.4-tcc)
      TAR_SRC=$(find_input tar-1.12-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$GZIP/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/tar-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      TAR_DIR=""
      for d in "$TAR_SRC"/tar-*; do
        if [ -d "$d" ]; then TAR_DIR="$d"; break; fi
      done
      $BB cp -r "$TAR_DIR" tar-1.12
      $BB chmod -R u+w tar-1.12
      cd tar-1.12

      # Create empty config.h
      : > config.h

      # Provide getdate_stub.c (avoids yacc/bison dependency)
      $BB cat > lib/getdate_stub.c << 'STUB'
#include <time.h>
time_t get_date(const char *p, const time_t *now) { return -1; }
STUB

      # Prepend stat override for tcc
      $BB cat > /tmp/stat_override.c << 'STAT_EOF'
#ifdef __TINYC__
#define st_atime st_atim.tv_sec
#define st_mtime st_mtim.tv_sec
#define st_ctime st_ctim.tv_sec
#endif
STAT_EOF
      $BB cat /tmp/stat_override.c src/create.c > src/create.c.new
      $BB mv src/create.c.new src/create.c

      CFLAGS="-I. -Ilib -DHAVE_FCNTL_H -DHAVE_DIRENT_H -DHAVE_GETCWD -DSIZEOF_UNSIGNED_LONG=4 -DVERSION=\\\"1.12\\\" -DPACKAGE=\\\"tar\\\""

      # Compile library sources
      for src in argmatch backupfile error fnmatch ftruncate getdate_stub \
                 getopt getopt1 getversion modechange msleep xgetcwd \
                 xmalloc xstrdup; do
        tcc -c $CFLAGS lib/$src.c -o lib/$src.o
      done

      # Create archive
      tcc -ar cr libtar.a \
        lib/argmatch.o lib/backupfile.o lib/error.o lib/fnmatch.o \
        lib/ftruncate.o lib/getdate_stub.o lib/getopt.o lib/getopt1.o \
        lib/getversion.o lib/modechange.o lib/msleep.o lib/xgetcwd.o \
        lib/xmalloc.o lib/xstrdup.o

      # Compile tar sources
      for src in arith buffer compare create delete extract incremen list \
                 mangle misc names open3 rtapelib tar update; do
        tcc -c $CFLAGS src/$src.c -o src/$src.o
      done

      # Link
      tcc -static \
        src/arith.o src/buffer.o src/compare.o src/create.o src/delete.o \
        src/extract.o src/incremen.o src/list.o src/mangle.o src/misc.o \
        src/names.o src/open3.o src/rtapelib.o src/tar.o src/update.o \
        -L. -ltar -o tar

      # Verify
      ./tar --version

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp tar "$out/bin/tar"
      $BB chmod 755 "$out/bin/tar"

      test -x "$out/bin/tar"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, gzip, tar_src],
} | crunch.Derivation
