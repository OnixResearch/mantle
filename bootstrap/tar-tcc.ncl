# Build GNU tar 1.12 using tinycc 0.9.27 + make 3.82.
#
# tar replaces the stage0-posix untar utility with a full-featured
# implementation that handles more archive formats.
# Adapted from live-bootstrap steps/tar-1.12/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in
let gzip = import "gzip-tcc.ncl" in

let tar_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/tar/tar-1.12.tar.gz",
  hash = "sha256-5LM4ezQqPKZECBwzpVjxpe923L358cGsdH6jHyCdpLY=",
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
      STAGE0=$(find_input stage0-posix)
      GZIP=$(find_input gzip-1.2.4-tcc)
      TAR_SRC=$(find_input tar-1.12-src)

      export PATH="$TCC/bin:$STAGE0/bin:$GZIP/bin:$PATH"

      WORK=/tmp/tar-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips single top-level dir
      $BB cp -r "$TAR_SRC" tar-1.12
      $BB chmod -R u+w tar-1.12
      cd tar-1.12

      # Create config.h with string-valued package metadata. Putting these
      # definitions in the header avoids fragile shell/TinyCC quoting of -D
      # values such as VERSION="1.12".
      $BB cat > config.h << 'CONFIG_H'
#define VERSION "1.12"
#define PACKAGE "tar"
CONFIG_H

      # Provide getdate_stub.c (avoids yacc/bison dependency)
      $BB cat > lib/getdate_stub.c << 'STUB'
#include <time.h>
time_t get_date(const char *p, const time_t *now) { return -1; }
STUB

      # Mes stat headers already expose the st_atime/st_mtime/st_ctime field
      # names expected by tar 1.12 on this TinyCC handoff.

      CFLAGS="-I. -Ilib -DHAVE_CONFIG_H -DSTDC_HEADERS -DHAVE_STRING_H -DHAVE_STDLIB_H -DHAVE_UNISTD_H -DHAVE_FCNTL_H -DHAVE_DIRENT_H -DHAVE_GETCWD -DSIZEOF_UNSIGNED_LONG=4"

      # Compile library sources. Compile from inside each source directory so
      # Mes-linked TinyCC writes objects with simple output names.
      (cd lib
        for src in argmatch backupfile error fnmatch ftruncate getdate_stub \
                   getopt getopt1 getversion modechange msleep xgetcwd \
                   xmalloc xstrdup; do
          tcc -c -o $src.o $CFLAGS -I.. -I. $src.c
        done
      )

      LIB_OBJS="lib/argmatch.o lib/backupfile.o lib/error.o lib/fnmatch.o lib/ftruncate.o lib/getdate_stub.o lib/getopt.o lib/getopt1.o lib/getversion.o lib/modechange.o lib/msleep.o lib/xgetcwd.o lib/xmalloc.o lib/xstrdup.o"

      # Compile tar sources
      (cd src
        for src in arith buffer compare create delete extract incremen list \
                   mangle misc names open3 rtapelib tar update; do
          tcc -c -o $src.o $CFLAGS -I.. -I../lib $src.c
        done
      )

      # Link. Copy objects to the build root because this Mes-linked TinyCC
      # handoff can fail to open link inputs with directory components even
      # when the files exist.
      $BB chmod u+rw src/*.o lib/*.o
      for o in src/*.o lib/*.o; do $BB cp "$o" .; done
      tcc -static \
        arith.o buffer.o compare.o create.o delete.o \
        extract.o incremen.o list.o mangle.o misc.o \
        names.o open3.o rtapelib.o tar.o update.o \
        argmatch.o backupfile.o error.o fnmatch.o \
        ftruncate.o getdate_stub.o getopt.o getopt1.o \
        getversion.o modechange.o msleep.o xgetcwd.o \
        xmalloc.o xstrdup.o -o tar
      $BB chmod 755 tar

      # Verify
      ./tar --version

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp tar "$out/bin/tar"
      $BB chmod 755 "$out/bin/tar"

      test -x "$out/bin/tar"
    "%,
  ],
  inputs = [stage0, tcc, gzip, tar_src],
} | crunch.Derivation
