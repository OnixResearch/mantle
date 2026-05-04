# Build GNU patch 2.5.9 using tinycc 0.9.27 + make 3.82.
#
# Adapted from live-bootstrap steps/patch-2.5.9/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in

let patch_src = crunch.fetchTarball {
  # first-consumer: bootstrap/patch-tcc.ncl
  # provenance: fosslinux/live-bootstrap steps/patch-2.5.9/pass1.kaem source input
  url = "https://ftpmirror.gnu.org/patch/patch-2.5.9.tar.gz",
  hash = "sha256-LiyweTmLg8H9PeHjwfJAPO34y//rF+mD+7IdsozDzLM=",
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

      # fetchTarball strips single top-level dir
      $BB cp -r "$PATCH_SRC" patch-2.5.9
      $BB chmod -R u+w patch-2.5.9
      cd patch-2.5.9

      # Keep string-valued configure substitutions in config.h instead of
      # passing quoted -D values through tcc-mes; its command-line define
      # parser rejects the escaped quotes before source compilation starts.
      $BB cat > config.h <<'CONFIG_H'
#define PACKAGE_BUGREPORT ""
#define PACKAGE_NAME "patch"
#define PACKAGE_VERSION "2.5.9"
#define PACKAGE "patch"
#define VERSION "2.5.9"
#define ed_PROGRAM "/nullop"
#define HAVE_DECL_GETENV 1
#define HAVE_DECL_MALLOC 1
#define HAVE_DECL_FREE 1
#define HAVE_DECL_MKTEMP 1
#define HAVE_DECL_STRERROR 1
#define HAVE_DECL_STRERROR_R 0
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_GETEUID 1
#define HAVE_MKTEMP 1
#define HAVE_MKDIR 1
#define HAVE_RMDIR 1
#define HAVE_FCNTL_H 1
#define HAVE_MALLOC 1
#define HAVE_REALLOC 1
#define HAVE_STRING_H 1
#define HAVE_STRINGS_H 1
#define HAVE_STDLIB_H 1
#define HAVE_UNISTD_H 1
#define HAVE_UTIME_H 0
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_STDINT_H 1
#define HAVE_STDDEF_H 1
#define HAVE_VPRINTF 1
#define HAVE_GETUID 1
#define HAVE_STRERROR 1
#define HAVE_MEMCHR 1
#define HAVE_MEMCMP 1
#define HAVE_LONG_FILE_NAMES 1
#define HAVE_STRUCT_UTIMBUF 0
#define PROTOTYPES 1
#define STDC_HEADERS 1
#define mbstate_t void*
#define RETSIGTYPE int
CONFIG_H
      : > patchlevel.h

      CFLAGS="-I. -DHAVE_CONFIG_H"

      # Compile each source
      for src in error getopt getopt1 addext argmatch backupfile basename \
                 dirname inp maketime partime patch pch quote quotearg \
                 quotesys util version xmalloc; do
        tcc -c $CFLAGS $src.c -o $src.o
        $BB chmod u+rw "$src.o"
        test -s "$src.o"
      done

      $BB chmod 644 *.o

      # Link
      tcc -L"$TCC/lib/mes" -L"$TCC/lib/mes/tcc" -static -o patch \
        error.o getopt.o getopt1.o addext.o argmatch.o backupfile.o \
        basename.o dirname.o inp.o maketime.o partime.o patch.o pch.o \
        quote.o quotearg.o quotesys.o util.o version.o xmalloc.o

      $BB chmod 755 patch

      # Verify
      ./patch --version
      $BB printf 'old\n' > smoke.txt
      $BB cat > smoke.diff <<'EOF'
--- smoke.txt
+++ smoke.txt
@@ -1 +1 @@
-old
+new
EOF
      ./patch smoke.txt < smoke.diff
      $BB grep '^new$' smoke.txt

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp patch "$out/bin/patch"
      $BB chmod 755 "$out/bin/patch"

      test -x "$out/bin/patch"
      "$out/bin/patch" --version
    "%,
  ],
  inputs = [stage0, mes, tcc, make, patch_src],
} | crunch.Derivation
