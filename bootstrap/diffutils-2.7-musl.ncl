# Build diffutils 2.7 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/diffutils-2.7.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc = crunch.derivationFile "tcc-musl-v2.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-4.0.9-musl.ncl" in

let diffutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/diffutils-2.7/sources
  # first-consumer: bootstrap/diffutils-2.7-musl.ncl
  url = "https://mirrors.kernel.org/gnu/diffutils/diffutils-2.7.tar.gz",
  hash = "sha256-AdsuvMN3btb0J5q9xxJs2zrSSO8+V7gd7yBMO4LVpxs=",
  name = "diffutils-2.7-src",
} in

{
  name = "diffutils-2.7-musl",
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
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      SRC=$(find_input diffutils-2.7-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/diffutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" diffutils-2.7
      $BB chmod -R u+w diffutils-2.7
      cd diffutils-2.7

      : > config.h
      CFLAGS="-I. -I$MUSL/include -DNULL_DEVICE=\"/dev/null\" -DHAVE_STRERROR=1 -DREGEX_MALLOC=1 -DHAVE_DIRENT_H=1 -DHAVE_DUP2=1 -DHAVE_FORK=1 -DHAVE_UNISTD_H=1 -DHAVE_STRING_H=1 -DHAVE_STDLIB_H=1"
      CMP_SOURCES='cmp cmpbuf error getopt getopt1 xmalloc version'
      DIFF_SOURCES='diff alloca analyze cmpbuf dir io util context ed ifdef normal side fnmatch getopt getopt1 regex version'
      compile_object() {
        source_path="$1"
        object_path="$2"
        "$TCC/bin/tcc" -c $CFLAGS "$source_path" -o "$object_path" || {
          $BB rm -f "$object_path"
          echo "ERROR: TCC-v2 failed compiling $source_path" >&2
          exit 1
        }
        test -s "$object_path"
        $BB chmod u+rw "$object_path"
      }

      compile_sources() {
        output_name=$1
        shift
        object_files=
        for source_name in "$@"; do
          object_path="$WORK/$output_name-$source_name.o"
          echo "diffutils-2.7-source=$source_name.c" >&2
          compile_object "$source_name.c" "$object_path"
          object_files="$object_files $object_path"
        done
        echo "diffutils-2.7-link=$output_name" >&2
        "$TCC/bin/tcc" -nostdlib -static -o "$WORK/$output_name" \
          "$MUSL/lib/crt1.o" $object_files "$MUSL/lib/libc.a"
        test -s "$WORK/$output_name"
        $BB chmod u+r "$WORK/$output_name"
      }

      # Keep the source lists explicit and bounded; these are the source sets
      # used by live-bootstrap's diffutils-2.7 pass1 make fragment.
      compile_sources cmp $CMP_SOURCES
      compile_sources diff $DIFF_SOURCES

      $BB mkdir -p "$out/bin"
      $BB cp "$WORK/diff" "$out/bin/diff"
      $BB cp "$WORK/cmp" "$out/bin/cmp"
      $BB chmod 755 "$out/bin/diff" "$out/bin/cmp"
      if $BB grep -q '^#!' "$out/bin/diff" "$out/bin/cmp"; then
        echo 'ERROR: source-built diffutils output is a script' >&2
        exit 1
      fi

      test -x "$out/bin/diff" || { echo "ERROR: installed diff missing" >&2; exit 1; }
      test -x "$out/bin/cmp" || { echo "ERROR: installed cmp missing" >&2; exit 1; }
      echo same > /tmp/diffutils-smoke-a
      echo same > /tmp/diffutils-smoke-b
      "$out/bin/diff" /tmp/diffutils-smoke-a /tmp/diffutils-smoke-b
      "$out/bin/cmp" /tmp/diffutils-smoke-a /tmp/diffutils-smoke-b
      echo changed > /tmp/diffutils-smoke-b
      if "$out/bin/cmp" /tmp/diffutils-smoke-a /tmp/diffutils-smoke-b >/tmp/diffutils-cmp.out 2>&1; then
        echo "ERROR: cmp did not detect different files" >&2; exit 1
      fi
      if "$out/bin/diff" /tmp/diffutils-smoke-a /tmp/diffutils-smoke-b >/tmp/diffutils-diff.out 2>&1; then
        echo "ERROR: diff did not detect different files" >&2; exit 1
      fi
      $BB grep -q changed /tmp/diffutils-diff.out
      if "$out/bin/diff" --mantle-invalid-option \
          >/tmp/diffutils-rejected-output 2>/tmp/diffutils-rejected-error; then
        echo "ERROR: diff accepted malformed input" >&2
        exit 1
      fi
      test ! -s /tmp/diffutils-rejected-output
      test -s /tmp/diffutils-rejected-error
      echo "diffutils 2.7 source-built cmp/diff runtime and rejection checks passed" >&2
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, diffutils_src],
} | crunch.Derivation
