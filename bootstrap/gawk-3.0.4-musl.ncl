# Build gawk 3.0.4 linked against musl using tcc-musl-v2.
#
# awk is needed by autoconf/automake for generated source processing.
# Adapted from live-bootstrap steps/gawk-3.0.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
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

      TCC=$(find_input tcc-0.9.27-musl-v2)
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
extern unsigned long mantle_double_to_ulong_bounded(double, int *);
CONFIG

      CFLAGS="-I. -I$MUSL/include -DHAVE_CONFIG_H -Dstrtod=mantle_strtod -Datof=mantle_atof"

      # Preserve the original decimal AWKNUM semantics while compiling with the
      # proven TinyCC-v2 shell. The explicit runtime checks below reject NaN or
      # corrupted numeric nodes before this Gawk can serve generators.
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

      # TinyCC-v2 encodes source-level integral floating literals as NaN at
      # this frontier, and native musl's TCC-built decimal conversion is also
      # corrupt. Normalize only exact 0.0/1.0/-1.0 literals in executable
      # numeric-node paths to AWKNUM
      # casts, preserving the double-typed semantics and failing closed on any
      # authenticated source-shape drift.
      EXPECTED_EVAL_INCREMENT_LITERAL_COUNT=2
      EXPECTED_EVAL_ZERO_LITERAL_COUNT=3
      EXPECTED_NODE_RETURN_ZERO_LITERAL_COUNT=3
      EXPECTED_IO_ZERO_RESULT_COUNT=4
      EXPECTED_IO_NEGATIVE_RESULT_COUNT=2
      EXPECTED_PARSER_ZERO_NODE_COUNT=7
      EXPECTED_BUILTIN_NEGATIVE_COMPARISON_COUNT=2

      test "$($BB grep -F -c '? 1.0 : -1.0' eval.c)" -eq "$EXPECTED_EVAL_INCREMENT_LITERAL_COUNT" || { echo "ERROR: unexpected Gawk increment literal shape" >&2; exit 1; }
      test "$($BB grep -F -c ' != 0.0' eval.c)" -eq "$EXPECTED_EVAL_ZERO_LITERAL_COUNT" || { echo "ERROR: unexpected Gawk condition-zero literal shape" >&2; exit 1; }
      $BB sed -i 's/? 1\.0 : -1\.0/? (AWKNUM)1 : (AWKNUM)-1/g; s/ != 0\.0/ != (AWKNUM)0/g' eval.c

      test "$($BB grep -F -c 'Nnull_string->numbr = 0.0;' main.c)" -eq 1 || { echo "ERROR: unexpected Gawk null-node literal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'tmp_number(0.0)' main.c)" -eq 1 || { echo "ERROR: unexpected Gawk ARGV-zero literal shape" >&2; exit 1; }
      $BB sed -i 's/Nnull_string->numbr = 0\.0;/Nnull_string->numbr = (AWKNUM)0;/; s/tmp_number(0\.0)/tmp_number((AWKNUM)0)/' main.c

      test "$($BB grep -F -c 'n->numbr = 0.0;' node.c)" -eq 1 || { echo "ERROR: unexpected Gawk forced-number assignment shape" >&2; exit 1; }
      test "$($BB grep -F -c 'return 0.0;' node.c)" -eq "$EXPECTED_NODE_RETURN_ZERO_LITERAL_COUNT" || { echo "ERROR: unexpected Gawk forced-number return shape" >&2; exit 1; }
      $BB sed -i 's/n->numbr = 0\.0;/n->numbr = (AWKNUM)0;/; s/return 0\.0;/return (AWKNUM)0;/g' node.c

      test "$($BB grep -F -c 'tmp_number((AWKNUM) 0.0)' io.c)" -eq "$EXPECTED_IO_ZERO_RESULT_COUNT" || { echo "ERROR: unexpected Gawk I/O zero-result shape" >&2; exit 1; }
      test "$($BB grep -F -c 'tmp_number((AWKNUM) -1.0)' io.c)" -eq "$EXPECTED_IO_NEGATIVE_RESULT_COUNT" || { echo "ERROR: unexpected Gawk I/O negative-result shape" >&2; exit 1; }
      test "$($BB grep -F -c 'tmp_number((AWKNUM) 1.0)' io.c)" -eq 1 || { echo "ERROR: unexpected Gawk I/O positive-result shape" >&2; exit 1; }
      $BB sed -i 's/tmp_number((AWKNUM) 0\.0)/tmp_number((AWKNUM)0)/g; s/tmp_number((AWKNUM) -1\.0)/tmp_number((AWKNUM)-1)/g; s/tmp_number((AWKNUM) 1\.0)/tmp_number((AWKNUM)1)/g' io.c

      test "$($BB grep -F -c 'make_number(0.0)' awktab.c)" -eq "$EXPECTED_PARSER_ZERO_NODE_COUNT" || { echo "ERROR: unexpected Gawk parser zero-node shape" >&2; exit 1; }
      $BB sed -i 's/make_number(0\.0)/make_number((AWKNUM)0)/g' awktab.c

      test "$($BB grep -F -c 'arg < 0.0' builtin.c)" -eq "$EXPECTED_BUILTIN_NEGATIVE_COMPARISON_COUNT" || { echo "ERROR: unexpected Gawk builtin negative-comparison shape" >&2; exit 1; }
      test "$($BB grep -F -c 'd_index < 1.0' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk index literal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'd_length <= 0.0' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk length literal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'rlength = -1.0' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk match-failure literal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'tmp_number((AWKNUM) 0.0)' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk builtin zero-result shape" >&2; exit 1; }
      test "$($BB grep -F -c 'g == 0.0' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk format-zero literal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'AWKNUM retval = 0.0' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk accumulator-zero literal shape" >&2; exit 1; }
      $BB sed -i 's/arg < 0\.0/arg < (AWKNUM)0/g; s/d_index < 1\.0/d_index < (AWKNUM)1/; s/d_length <= 0\.0/d_length <= (AWKNUM)0/; s/rlength = -1\.0/rlength = (AWKNUM)-1/; s/tmp_number((AWKNUM) 0\.0)/tmp_number((AWKNUM)0)/; s/g == 0\.0/g == (AWKNUM)0/; s/AWKNUM retval = 0\.0/AWKNUM retval = (AWKNUM)0/' builtin.c

      # TCC-v2's old-style __fixunsdfdi call ABI returns zero in Gawk's
      # formatter. Route only the two positive and two negative integer-format
      # conversions through the bounded binary-decomposition core.
      EXPECTED_FORMAT_UNSIGNED_BOUND_COUNT=2
      EXPECTED_FORMAT_POSITIVE_CONVERSION_COUNT=2
      EXPECTED_FORMAT_NEGATIVE_DECIMAL_COUNT=1
      EXPECTED_FORMAT_NEGATIVE_HEX_COUNT=1
      test "$($BB grep -F -c 'tmpval <= (unsigned long) ULONG_MAX' builtin.c)" -eq "$EXPECTED_FORMAT_UNSIGNED_BOUND_COUNT" || { echo "ERROR: unexpected Gawk format unsigned-bound shape" >&2; exit 1; }
      test "$($BB grep -F -c 'uval = (unsigned long) tmpval;' builtin.c)" -eq "$EXPECTED_FORMAT_POSITIVE_CONVERSION_COUNT" || { echo "ERROR: unexpected Gawk format positive-conversion shape" >&2; exit 1; }
      test "$($BB grep -F -c 'uval = - (unsigned long) (long) tmpval;' builtin.c)" -eq "$EXPECTED_FORMAT_NEGATIVE_DECIMAL_COUNT" || { echo "ERROR: unexpected Gawk format negative-decimal shape" >&2; exit 1; }
      test "$($BB grep -F -c 'uval = (unsigned long) (long) tmpval;' builtin.c)" -eq "$EXPECTED_FORMAT_NEGATIVE_HEX_COUNT" || { echo "ERROR: unexpected Gawk format negative-hex shape" >&2; exit 1; }
      test "$($BB grep -F -c 'int sgn;' builtin.c)" -eq 1 || { echo "ERROR: unexpected Gawk formatter local shape" >&2; exit 1; }
      $BB sed -i 's/int sgn;/int sgn; int mantle_conversion_ok;/; s/if (! (tmpval <= (unsigned long) ULONG_MAX))/if (0)/g; s/uval = - (unsigned long) (long) tmpval;/uval = mantle_double_to_ulong_bounded(-tmpval, \&mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;/; s/uval = (unsigned long) (long) tmpval;/uval = mantle_double_to_ulong_bounded(tmpval, \&mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;/; s/uval = (unsigned long) tmpval;/uval = mantle_double_to_ulong_bounded(tmpval, \&mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;/g' builtin.c

      if $BB grep -F '? 1.0 : -1.0' eval.c \
        || $BB grep -F 'Nnull_string->numbr = 0.0;' main.c \
        || $BB grep -F 'return 0.0;' node.c \
        || $BB grep -E 'tmp_number\(\(AWKNUM\) [+-]?[01]\.0\)' io.c builtin.c \
        || $BB grep -F 'make_number(0.0)' awktab.c; then
        echo "ERROR: unsupported integral floating literal remained in Gawk numeric-node paths" >&2
        exit 1
      fi

      # The native-musl compatibility runtime's TCC-built strtod/atof path
      # returns NaN for ordinary decimal text at this frontier. Keep libc and
      # startup ownership in native musl, but compile a bounded decimal parser
      # with the declared TCC-v2 and route only Gawk's conversion calls to it.
      $BB cat > mantle_decimal.c <<'DECIMAL_EOF'
#include <stddef.h>
#include <limits.h>

#define MANTLE_DECIMAL_INPUT_BYTES_MAX 4096
#define MANTLE_DECIMAL_SIGNIFICAND_DIGITS_MAX 64
#define MANTLE_DECIMAL_SCALE_STEPS_MAX 512
#define MANTLE_DECIMAL_RADIX 10

/* Override only the two broken unsigned conversion helpers from the
 * authenticated predecessor libtcc1.a. Signed 64-bit conversions emitted by
 * TCC-v2 are sound at this frontier and form the bounded implementation core.
 */
double
__floatundidf(value)
unsigned long value;
{
    double upper;

    if (value <= (unsigned long) LONG_MAX)
        return (double) (long) value;
    upper = (double) (long) (value >> 1);
    return upper * (double) 2 + (double) (value & (unsigned long) 1);
}

unsigned long
mantle_double_to_ulong_bounded(value, valid_out)
double value;
int *valid_out;
{
    double remaining;
    double unit = (double) 1;
    unsigned long bit = (unsigned long) 1;
    unsigned long result = (unsigned long) 0;
    int negative;
    int magnitude_bit_count = sizeof(unsigned long) * CHAR_BIT - 1;
    int step_count = 1;

    *valid_out = 0;
    if (value != value)
        return result;
    negative = value < (double) 0;
    remaining = negative ? -value : value;
    while (step_count < magnitude_bit_count) {
        unit *= (double) 2;
        bit <<= 1;
        step_count++;
    }
    while (bit != (unsigned long) 0) {
        if (remaining >= unit) {
            remaining -= unit;
            result |= bit;
        }
        unit /= (double) 2;
        bit >>= 1;
    }
    if (remaining >= (double) 1)
        return (unsigned long) 0;
    if (negative)
        result = (unsigned long) 0 - result;
    *valid_out = 1;
    return result;
}

unsigned long
__fixunsdfdi(value)
double value;
{
    int valid = 0;
    unsigned long result = mantle_double_to_ulong_bounded(value, &valid);

    return valid ? result : (unsigned long) 0;
}

static int
mantle_decimal_is_space(c)
int c;
{
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v';
}

static int
mantle_decimal_is_digit(c)
int c;
{
    return c >= '0' && c <= '9';
}

static double
mantle_decimal_apply_scale(value, decimal_adjustment)
double value;
int decimal_adjustment;
{
    int step_count = 0;

    while (decimal_adjustment > 0 && step_count < MANTLE_DECIMAL_SCALE_STEPS_MAX) {
        value *= (double) MANTLE_DECIMAL_RADIX;
        decimal_adjustment--;
        step_count++;
    }
    while (decimal_adjustment < 0 && step_count < MANTLE_DECIMAL_SCALE_STEPS_MAX) {
        value /= (double) MANTLE_DECIMAL_RADIX;
        decimal_adjustment++;
        step_count++;
    }
    return value;
}

static int
mantle_decimal_exponent(cursor, input_end, parsed_end, exponent_out)
const char *cursor;
const char *input_end;
const char **parsed_end;
int *exponent_out;
{
    const char *scan = cursor;
    int exponent = 0;
    int sign = 1;
    int saw_digit = 0;

    if (scan < input_end && (*scan == '+' || *scan == '-')) {
        if (*scan == '-')
            sign = -1;
        scan++;
    }
    while (scan < input_end && mantle_decimal_is_digit(*scan)) {
        saw_digit = 1;
        if (exponent < MANTLE_DECIMAL_SCALE_STEPS_MAX)
            exponent = exponent * MANTLE_DECIMAL_RADIX + (*scan - '0');
        if (exponent > MANTLE_DECIMAL_SCALE_STEPS_MAX)
            exponent = MANTLE_DECIMAL_SCALE_STEPS_MAX;
        scan++;
    }
    if (! saw_digit)
        return 0;
    *parsed_end = scan;
    *exponent_out = sign * exponent;
    return 1;
}

static double
mantle_decimal_parse(text, endptr)
const char *text;
char **endptr;
{
    const char *cursor = text;
    const char *input_end;
    const char *exponent_end;
    double value = (double) 0;
    int sign = 1;
    int saw_digit = 0;
    int saw_decimal = 0;
    int kept_digits = 0;
    int decimal_adjustment = 0;
    int explicit_exponent = 0;

    if (text == NULL) {
        if (endptr != NULL)
            *endptr = NULL;
        return (double) 0;
    }
    input_end = text + MANTLE_DECIMAL_INPUT_BYTES_MAX;
    while (cursor < input_end && mantle_decimal_is_space(*cursor))
        cursor++;
    if (cursor < input_end && (*cursor == '+' || *cursor == '-')) {
        if (*cursor == '-')
            sign = -1;
        cursor++;
    }
    while (cursor < input_end) {
        if (! saw_decimal && *cursor == '.') {
            saw_decimal = 1;
            cursor++;
            continue;
        }
        if (! mantle_decimal_is_digit(*cursor))
            break;
        saw_digit = 1;
        if (kept_digits < MANTLE_DECIMAL_SIGNIFICAND_DIGITS_MAX) {
            value = value * (double) MANTLE_DECIMAL_RADIX + (double) (*cursor - '0');
            kept_digits++;
            if (saw_decimal)
                decimal_adjustment--;
        } else if (! saw_decimal) {
            decimal_adjustment++;
        }
        cursor++;
    }
    if (! saw_digit) {
        if (endptr != NULL)
            *endptr = (char *) text;
        return (double) 0;
    }
    if (cursor < input_end && (*cursor == 'e' || *cursor == 'E')) {
        if (mantle_decimal_exponent(cursor + 1, input_end, &exponent_end, &explicit_exponent))
            cursor = exponent_end;
    }
    decimal_adjustment += explicit_exponent;
    value = mantle_decimal_apply_scale(value, decimal_adjustment);
    if (endptr != NULL)
        *endptr = (char *) cursor;
    return sign < 0 ? -value : value;
}

double
mantle_strtod(text, endptr)
const char *text;
char **endptr;
{
    return mantle_decimal_parse(text, endptr);
}

double
mantle_atof(text)
const char *text;
{
    return mantle_decimal_parse(text, (char **) NULL);
}
DECIMAL_EOF
      test "$($BB grep -F -c '#define MANTLE_DECIMAL_INPUT_BYTES_MAX 4096' mantle_decimal.c)" -eq 1 || { echo "ERROR: bounded Gawk decimal parser missing" >&2; exit 1; }
      test "$($BB grep -F -c 'mantle_decimal_parse(text' mantle_decimal.c)" -eq 3 || { echo "ERROR: unexpected Gawk decimal parser call shape" >&2; exit 1; }
      test "$($BB grep -F -c '__floatundidf(value)' mantle_decimal.c)" -eq 1 || { echo "ERROR: bounded unsigned-to-double helper missing" >&2; exit 1; }
      test "$($BB grep -F -c '__fixunsdfdi(value)' mantle_decimal.c)" -eq 1 || { echo "ERROR: bounded double-to-unsigned helper missing" >&2; exit 1; }

      OBJS=""
      for src in mantle_decimal array builtin dfa eval field io main msg node random re regex version; do
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
        # Compile parser numeric actions with the same declared TinyCC-v2
        # compiler and reject corrupted decimal behavior after linking.
        "$TCC/bin/tcc" -c $CFLAGS -Dalloca=malloc awktab.c -o awktab.o || { echo "ERROR: failed compiling awktab.c" >&2; exit 1; }
        test -f awktab.o || { echo "ERROR: missing parser object awktab.o" >&2; exit 1; }
        $BB chmod u+rw awktab.o
        OBJS="$OBJS awktab.o"
      else
        echo "ERROR: no pregenerated gawk parser source found" >&2
        exit 1
      fi
      test -n "$OBJS" || { echo "ERROR: no gawk objects were built" >&2; exit 1; }

      # TinyCC-v2 emits two x86_64 unsigned integer/float conversion calls.
      # mantle_decimal.o owns their bounded implementations, so no predecessor
      # compiler runtime or Mes libc enters this link; native musl owns startup
      # and libc.
      tcc -nostdlib -static -o gawk \
        "$MUSL/lib/crt1.o" $OBJS "$MUSL/lib/libc.a" || {
          echo "ERROR: failed linking gawk against native musl" >&2
          exit 1
        }
      $BB chmod 755 gawk
      test -x gawk || { echo "ERROR: linked gawk missing" >&2; exit 1; }

      EXPECTED_TOTAL=7
      EXPECTED_MATCHES=2
      EXPECTED_DECIMAL_PRODUCT=3
      EXPECTED_STRING_LENGTH=2
      HEX_INPUT_DECIMAL=68
      EXPECTED_HEX=00000044
      HEX_FORMAT=$($BB printf '\04508x')
      printf 'alpha\nbeta\n' | LC_ALL=C ./gawk \
        'BEGIN { label = toupper("awk"); total = 2 + 5 } /a/ { matches++ } END { print total ":" matches ":" label }' \
        > "$WORK/gawk-smoke.out"
      test "$($BB cat "$WORK/gawk-smoke.out")" = "$EXPECTED_TOTAL:$EXPECTED_MATCHES:AWK" || {
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
      DECIMAL_OUTPUT=$(LC_ALL=C ./gawk 'BEGIN { product = 1.5 * 2.0; print product }')
      test "$DECIMAL_OUTPUT" = "$EXPECTED_DECIMAL_PRODUCT" || {
        echo "gawk-decimal-output=$DECIMAL_OUTPUT" >&2
        echo "ERROR: gawk decimal arithmetic smoke failed" >&2
        exit 1
      }
      LENGTH_OUTPUT=$(LC_ALL=C ./gawk 'BEGIN { print length("S?") }')
      test "$LENGTH_OUTPUT" = "$EXPECTED_STRING_LENGTH" || {
        echo "gawk-length-output=$LENGTH_OUTPUT" >&2
        echo "ERROR: gawk generator string-length smoke failed" >&2
        exit 1
      }
      HEX_OUTPUT=$(LC_ALL=C ./gawk -v value="$HEX_INPUT_DECIMAL" -v format="$HEX_FORMAT" 'BEGIN { printf format "\n", value }')
      test "$HEX_OUTPUT" = "$EXPECTED_HEX" || {
        echo "gawk-hex-output=$HEX_OUTPUT" >&2
        echo "ERROR: gawk generator hexadecimal-format smoke failed" >&2
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
