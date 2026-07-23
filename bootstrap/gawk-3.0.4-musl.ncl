# Build gawk 3.0.4 linked against musl using tcc-musl-v2.
#
# awk is needed by autoconf/automake for generated source processing.
# Adapted from live-bootstrap steps/gawk-3.0.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-native.ncl" in
let musl = import "musl-1.1.24-native.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let gawk_src = import "gawk-3.0.4-src.ncl" in

{
  name = "gawk-3.0.4-musl",
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

      TCC=$(find_input tcc-0.9.27-musl-native-runtime)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      SRC=$(find_input gawk-3.0.4-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin"

      WORK=/tmp/gawk-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" gawk-3.0.4
      $BB chmod -R u+w gawk-3.0.4
      cd gawk-3.0.4

      $BB cat > config.h << 'CONFIG'
#define STDC_HEADERS 1
#define HAVE_ALLOCA_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_FCNTL_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_MEMORY_H 1
#define HAVE_SYS_PARAM_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_WAIT_H 1
#define HAVE_STDARG_H 1
#define HAVE_VPRINTF 1
#define HAVE_SETLOCALE 1
#define HAVE_MEMCMP 1
#define HAVE_MEMCPY 1
#define HAVE_MEMSET 1
#define HAVE_STRCHR 1
#define HAVE_STRERROR 1
#define HAVE_STRFTIME 1
#define HAVE_STRNCASECMP 1
#define HAVE_STRTOD 1
#define HAVE_SYSTEM 1
#define HAVE_TZSET 1
#define HAVE_STRINGIZE 1
#define REGEX_MALLOC 1
#define SPRINTF_RET int
#define GETPGRP_VOID 1
#define GETGROUPS_T int
#define RETSIGTYPE void
#define VERSION "3.0.4"
#define PACKAGE "gawk"
#define PACKAGE_NAME "GNU Awk"
#define PACKAGE_VERSION "3.0.4"
#define DEFPATH ".:/usr/share/awk"
CONFIG

      CFLAGS="-I. -I$MUSL/include -DHAVE_CONFIG_H"

      # Compile the original decimal AWKNUM constants with the native-runtime
      # TinyCC.  The earlier integer-cast rewrite built a runnable binary but
      # corrupted AWK numeric nodes under real GCC generator workloads.
      $BB sed -i 's/#define Floor(n).*/#define Floor(n) floor(n)/; s/#define Ceil(n).*/#define Ceil(n) ceil(n)/' builtin.c
      # TinyCC miscompiles the double-to-LONG_MAX comparison as true for small
      # values.  Keep the integral-value check and omit only the out-of-range
      # optimization guard; GCC generator indices stay bounded, while large
      # AWKNUM formatting remains an explicit non-claim before admission.
      $BB sed -i 's/if ((val = double_to_int(s->numbr)) != s->numbr$/if ((val = double_to_int(s->numbr)) != s->numbr) {/' node.c
      $BB sed -i '/|| val < LONG_MIN || val > LONG_MAX) {/d' node.c
      $BB grep -q 'if ((val = double_to_int(s->numbr)) != s->numbr) {' node.c
      # Disable gawk's single-digit pointer-table micro-optimization so the
      # GCC workload exercises ordinary decimal integer formatting directly.
      $BB sed -i '/if (num < NVAL && num >= 0) {/,/^[[:space:]]*}[[:space:]]*$/c\
\t\t(void) sprintf(sp, "\\045ld", num);\
\t\ts->stlen = strlen(sp);' node.c
      $BB grep -q 'sprintf(sp,' node.c

      OBJS=""
      for src in array builtin dfa eval field io main msg node random re regex version; do
        obj="$src.o"
        "$TCC/bin/tcc" -c $CFLAGS "$src.c" -o "$obj" || { echo "ERROR: failed compiling $src.c" >&2; exit 1; }
        test -f "$obj" || { echo "ERROR: missing object $obj" >&2; exit 1; }
        $BB chmod u+rw "$obj"
        OBJS="$OBJS $obj"
      done

      "$TCC/bin/tcc" -c $CFLAGS gawkmisc.c -o gawkmisc.o || { echo "ERROR: failed compiling gawkmisc.c" >&2; exit 1; }
      test -f gawkmisc.o || { echo "ERROR: missing object gawkmisc.o" >&2; exit 1; }
      $BB chmod u+rw gawkmisc.o
      OBJS="$OBJS gawkmisc.o"

      # Use a pre-generated parser and fail closed if neither expected parser is present.
      if [ -f awkgram.c ]; then
        tcc -c $CFLAGS awkgram.c -o awkgram.o || { echo "ERROR: failed compiling awkgram.c" >&2; exit 1; }
        test -f awkgram.o || { echo "ERROR: missing parser object awkgram.o" >&2; exit 1; }
        $BB chmod u+rw awkgram.o
        OBJS="$OBJS awkgram.o"
      elif [ -f awktab.c ]; then
        # Compile parser numeric actions with the native-runtime TinyCC.  The
        # older predecessor encoded decimal AWK literals as NaNs, so a parser
        # object from that compiler cannot satisfy GCC generator semantics.
        "$TCC/bin/tcc" -c $CFLAGS -Dalloca=malloc awktab.c -o awktab.o || { echo "ERROR: failed compiling awktab.c" >&2; exit 1; }
        test -f awktab.o || { echo "ERROR: missing parser object awktab.o" >&2; exit 1; }
        $BB chmod u+rw awktab.o
        OBJS="$OBJS awktab.o"
      else
        echo "ERROR: no pregenerated gawk parser source found" >&2
        exit 1
      fi
      test -n "$OBJS" || { echo "ERROR: no gawk objects were built" >&2; exit 1; }

      tcc -nostdlib -static -o gawk \
        "$MUSL/lib/crt1.o" $OBJS "$MUSL/lib/libc.a" \
        "$TCC/lib/tcc/libtcc1.a" || {
          echo "ERROR: failed linking gawk against native musl" >&2
          exit 1
        }
      $BB chmod 755 gawk
      test -x gawk || { echo "ERROR: linked gawk missing" >&2; exit 1; }

      EXPECTED_TOTAL=7
      EXPECTED_MATCHES=2
      printf 'alpha\nbeta\n' | LC_ALL=C ./gawk \
        -v expected_total="$EXPECTED_TOTAL" -v expected_matches="$EXPECTED_MATCHES" \
        'BEGIN { label = toupper("awk"); total = 2 + 5 } /a/ { matches++ } END { if (total != expected_total || matches != expected_matches) exit 23; print label "-ok" }' \
        > "$WORK/gawk-smoke.out"
      test "$($BB cat "$WORK/gawk-smoke.out")" = "AWK-ok" || {
        echo "gawk-smoke-output=$($BB cat "$WORK/gawk-smoke.out")" >&2
        echo "ERROR: local gawk behavior smoke failed" >&2
        exit 1
      }
      NUMERIC_OUTPUT=$(LC_ALL=C ./gawk 'BEGIN { n = 0; values[++n] = "x"; print n ":" values[1] }')
      test "$NUMERIC_OUTPUT" = "1:x" || {
        echo "gawk-numeric-output=$NUMERIC_OUTPUT" >&2
        echo "ERROR: gawk numeric-array smoke failed" >&2
        exit 1
      }
      if LC_ALL=C ./gawk 'BEGIN {' > "$WORK/gawk-negative.out" 2>&1; then
        echo "ERROR: invalid gawk program unexpectedly succeeded" >&2
        exit 1
      fi
      $BB grep -q . "$WORK/gawk-negative.out" || {
        echo "ERROR: invalid gawk program emitted no diagnostic" >&2
        exit 1
      }

      $BB mkdir -p "$out/bin"
      $BB cp gawk "$out/bin/gawk"
      $BB chmod 755 "$out/bin/gawk"
      cd "$out/bin"
      $BB ln -s gawk awk

      test -x "$out/bin/gawk" || { echo "ERROR: installed gawk missing" >&2; exit 1; }
      test -L "$out/bin/awk" || { echo "ERROR: installed awk symlink missing" >&2; exit 1; }
      echo 'test' | "$out/bin/gawk" '{print "awk-ok"}' | $BB grep -q 'awk-ok' || { echo "ERROR: installed gawk smoke failed" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, gawk_src],
} | crunch.Derivation
