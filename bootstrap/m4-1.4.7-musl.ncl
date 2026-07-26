# Build genuine GNU M4 1.4.7 with the source-built TinyCC compiler shell and
# the separately qualified native-musl runtime candidate.
#
# Adapted from fosslinux/live-bootstrap steps/m4-1.4.7/mk/main.mk. The release
# sources are compiled directly, so no generated parser, host compiler, awk
# macro bridge, or ambient tool discovery participates in the provider.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-native.ncl" in

let m4_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/m4-1.4.7/sources
  # first-consumer: bootstrap/m4-1.4.7-musl.ncl
  url = "https://mirrors.kernel.org/gnu/m4/m4-1.4.7.tar.bz2",
  hash = "sha256-4SO98HrHjkV6a+NazK3EatKsynMNhfWXUpi1L5SeMMc=",
  name = "m4-1.4.7-src",
} in

{
  name = "m4-1.4.7-musl",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then
            echo "$candidate"
            return
          fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input m4-1.4.7-src)
      export PATH="$TCC/bin:$STAGE0/bin"

      WORK=/tmp/m4-1.4.7-musl
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      $BB cp -r "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"

      $BB cat > lib/config.h <<'EOF'
#define VERSION "1.4.7"
#define PACKAGE_BUGREPORT "bug-m4@gnu.org"
#define PACKAGE_STRING "GNU M4 1.4.7"
#define PACKAGE "m4"
#define PACKAGE_NAME "GNU M4"
#define HAVE_STDINT_H 1
#define HAVE___FPENDING 1
#define HAVE_DECL___FPENDING 1
#ifndef SIZE_MAX
#define SIZE_MAX ((size_t)-1)
#endif
#define _GNU_SOURCE 1
#define _GL_UNUSED
#define __getopt_argv_const const
#define SYSCMD_SHELL "/bin/sh"
EOF

      CFLAGS="-Ilib -Isrc -I$MUSL/include -DHAVE_CONFIG_H=1 -DSIZE_MAX=~(size_t)0"
      LIB_SOURCES="cloexec close-stream dup-safer error exitfail fd-safer fopen-safer getopt getopt1 mkstemp-safer regex obstack tmpfile-safer verror xalloc-die xasprintf xmalloc xvasprintf"
      M4_SOURCES="m4 builtin debug eval format freeze input macro output path symtab"
      OBJECTS=""
      LIB_SOURCE_COUNT=18
      M4_SOURCE_COUNT=11
      COMPILED_SOURCE_COUNT=0

      for source_name in $LIB_SOURCES; do
        object_path="lib/$source_name.o"
        tcc -c $CFLAGS "lib/$source_name.c" -o "$object_path" || {
          echo "ERROR: failed compiling GNU M4 library source lib/$source_name.c" >&2
          exit 1
        }
        test -s "$object_path" || {
          echo "ERROR: missing GNU M4 library object $object_path" >&2
          exit 1
        }
        OBJECTS="$OBJECTS $object_path"
        COMPILED_SOURCE_COUNT=$((COMPILED_SOURCE_COUNT + 1))
      done

      for source_name in $M4_SOURCES; do
        object_path="src/$source_name.o"
        tcc -c $CFLAGS "src/$source_name.c" -o "$object_path" || {
          echo "ERROR: failed compiling GNU M4 program source src/$source_name.c" >&2
          exit 1
        }
        test -s "$object_path" || {
          echo "ERROR: missing GNU M4 program object $object_path" >&2
          exit 1
        }
        OBJECTS="$OBJECTS $object_path"
        COMPILED_SOURCE_COUNT=$((COMPILED_SOURCE_COUNT + 1))
      done

      EXPECTED_SOURCE_COUNT=$((LIB_SOURCE_COUNT + M4_SOURCE_COUNT))
      test "$COMPILED_SOURCE_COUNT" -eq "$EXPECTED_SOURCE_COUNT" || {
        echo "ERROR: GNU M4 compiled-source count mismatch" >&2
        exit 1
      }

      # Compiler and runtime roles are explicit: the proven TinyCC-v2 shell
      # compiles and links, while native musl supplies startup and libc. No
      # TinyCC Mes runtime or stripped compatibility libc is inherited.
      GNU_M4_BINARY="$WORK/genuine-m4"
      tcc -nostdlib -static -o "$GNU_M4_BINARY" \
        "$MUSL/lib/crt1.o" $OBJECTS "$MUSL/lib/libc.a" || {
          echo "ERROR: failed linking genuine GNU M4" >&2
          exit 1
        }
      test -s "$GNU_M4_BINARY" || { echo "ERROR: linked GNU M4 binary missing" >&2; exit 1; }
      $BB chmod 755 "$GNU_M4_BINARY"

      $BB mkdir -p "$out/bin"
      $BB cp "$GNU_M4_BINARY" "$out/bin/m4"
      $BB chmod 755 "$out/bin/m4"
      test -x "$out/bin/m4" || { echo "ERROR: installed GNU M4 missing" >&2; exit 1; }

      "$out/bin/m4" --version > "$WORK/version.txt"
      $BB grep -q 'GNU M4 1.4.7' "$WORK/version.txt" || {
        echo "ERROR: GNU M4 version output mismatch" >&2
        exit 1
      }

      $BB cat > "$WORK/macro-positive.m4" <<'EOF'
define(`mantle_value', `42')mantle_value
EOF
      MACRO_OUTPUT=$("$out/bin/m4" "$WORK/macro-positive.m4")
      test "$MACRO_OUTPUT" = 42 || {
        echo "ERROR: GNU M4 ordinary macro expansion failed" >&2
        exit 1
      }

      # Flex invokes `m4 -P`; prove the prefixed builtin namespace and prove
      # that the unprefixed builtin is not silently accepted in that mode.
      $BB cat > "$WORK/prefix-positive.m4" <<'EOF'
m4_define(`mantle_prefixed_value', `prefixed-ok')mantle_prefixed_value
EOF
      PREFIX_OUTPUT=$("$out/bin/m4" -P "$WORK/prefix-positive.m4")
      test "$PREFIX_OUTPUT" = prefixed-ok || {
        echo "ERROR: GNU M4 -P prefixed macro expansion failed" >&2
        exit 1
      }
      $BB cat > "$WORK/prefix-negative.m4" <<'EOF'
define(`mantle_unprefixed_value', `must-not-expand')mantle_unprefixed_value
EOF
      PREFIX_NEGATIVE_OUTPUT=$("$out/bin/m4" -P "$WORK/prefix-negative.m4")
      test "$PREFIX_NEGATIVE_OUTPUT" != must-not-expand || {
        echo "ERROR: GNU M4 -P accepted the unprefixed define builtin" >&2
        exit 1
      }

      $BB cat > "$WORK/malformed-negative.m4" <<'EOF'
define(`unterminated, `value')
EOF
      if "$out/bin/m4" "$WORK/malformed-negative.m4" > "$WORK/malformed.out" 2> "$WORK/malformed.err"; then
        echo "ERROR: GNU M4 accepted malformed quoted input" >&2
        exit 1
      fi
      test -s "$WORK/malformed.err" || {
        echo "ERROR: GNU M4 malformed-input rejection emitted no diagnostic" >&2
        exit 1
      }
    "%
  ],
  inputs = [stage0, tcc, musl, m4_src],
} | crunch.Derivation
