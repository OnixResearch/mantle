# Build GNU patch 2.5.9 using tinycc 0.9.27 + make 3.82.
#
# Adapted from live-bootstrap steps/patch-2.5.9/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in

let patch_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/patch/patch-2.5.9.tar.gz",
  hash = "sha256-7LXGRp1zK88B1uwa/p5k8WaMq6W/2xA8KNf1N7o824o=",
  name = "patch-2.5.9-src",
} in

{
  name = "patch-2.5.9-tcc",
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
      PATCH_SRC=$(find_input patch-2.5.9-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/patch-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      PATCH_DIR=""
      for d in "$PATCH_SRC"/patch-*; do
        if [ -d "$d" ]; then PATCH_DIR="$d"; break; fi
      done
      $BB cp -r "$PATCH_DIR" patch-2.5.9
      $BB chmod -R u+w patch-2.5.9
      cd patch-2.5.9

      # Create empty config headers
      : > config.h
      : > patchlevel.h

      CFLAGS="-I. -DHAVE_DECL_GETENV -DHAVE_DECL_MALLOC -DHAVE_DIRENT_H -DHAVE_LIMITS_H -DHAVE_GETEUID -DHAVE_MKTEMP -DPACKAGE_BUGREPORT= -Ded_PROGRAM=\\\"/nullop\\\" -Dmbstate_t=void* -DRETSIGTYPE=int -DHAVE_MKDIR -DHAVE_RMDIR -DHAVE_FCNTL_H -DPACKAGE_NAME=\\\"patch\\\" -DPACKAGE_VERSION=\\\"2.5.9\\\" -DHAVE_MALLOC -DHAVE_REALLOC -DSTDC_HEADERS -DHAVE_STRING_H -DHAVE_STDLIB_H"

      # Compile each source
      for src in error getopt getopt1 addext argmatch backupfile basename \
                 dirname inp maketime partime patch pch quote quotearg \
                 quotesys util version xmalloc; do
        tcc -c $CFLAGS $src.c -o $src.o
      done

      # Link
      tcc -static \
        error.o getopt.o getopt1.o addext.o argmatch.o backupfile.o \
        basename.o dirname.o inp.o maketime.o partime.o patch.o pch.o \
        quote.o quotearg.o quotesys.o util.o version.o xmalloc.o \
        -o patch

      # Verify
      ./patch --version

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp patch "$out/bin/patch"
      $BB chmod 755 "$out/bin/patch"

      test -x "$out/bin/patch"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, patch_src],
} | crunch.Derivation
