# Rebuild the source-built TinyCC front end against the native-musl handoff.
#
# The preceding compiler runs on Mes libc, whose floating conversion is not
# sufficient for gawk.  This stage recompiles the patched TinyCC source with a
# native-musl va_list boundary, supplies a bounded source-literal parser, links
# against native musl, and proves positive and negative compiler behavior.  It
# remains an isolated predecessor until the GCC-built admission rails pass.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc26 = crunch.derivationFile "tinycc-mes.ncl" in
let host_tcc = crunch.derivationFile "tcc-musl-v2.ncl" in
let selfhost = crunch.derivationFile "tcc-musl-selfhost.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let binutils = crunch.derivationFile "binutils-tcc.ncl" in
let tcc27_src = crunch.fetchTarball {
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tcc-0.9.27-musl-native-runtime",
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

      TCC26=$(find_input tinycc-0.9.26)
      HOST_TCC=$(find_input tcc-0.9.27-musl-v2)
      SELFHOST=$(find_input tcc-0.9.27-musl-selfhost)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      BINUTILS=$(find_input binutils-2.30-tcc-source-v1)
      TCC27_SRC=$(find_input tcc-0.9.27-src)
      PATCHED_SOURCE="$SELFHOST/share/tcc-source-patched"
      test -f "$PATCHED_SOURCE/tcc.c" || { echo "ERROR: patched TinyCC source missing" >&2; exit 1; }

      WORK=/tmp/tcc-native-runtime
      $BB mkdir -p "$WORK/native-abi" "$out/bin" "$out/lib/tcc/include"
      $BB cp -r "$PATCHED_SOURCE" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"

      # Native musl's reduced formatted-I/O path cannot yet consume every
      # nested TinyCC diagnostic va_list.  Preserve deterministic error classes
      # by emitting the format literal without interpolated arguments.  This
      # avoids hiding frontend failures behind a diagnostic segfault, but the
      # full formatter remains an admission requirement for the GCC-built pass.
      $BB sed -i '/static void strcat_vprintf(char \*buf, int buf_size, const char \*fmt, va_list ap)/,/^}/c\
static void strcat_vprintf(char *buf, int buf_size, const char *fmt, va_list ap)\
{\
    int len = strlen(buf);\
    (void)ap;\
    while (*fmt && len < buf_size - 1) buf[len++] = *fmt++;\
    buf[len] = 0;\
}' libtcc.c
      # The reduced varargs and setjmp/longjmp paths corrupt diagnostics before
      # the CLI can reject bad source.  This frontier compiler processes one
      # translation unit per process, so use fixed messages, count nonfatal
      # errors, and fail closed on fatal errors.  Full diagnostics and recovery
      # remain admission requirements.
      $BB sed -i '/PUB_FUNC void tcc_error_noabort(const char \*fmt, ...)/,/^}/c\
PUB_FUNC void tcc_error_noabort(const char *fmt, ...)\
{\
    (void)fmt;\
    write(2, "tcc: nonfatal compile error\\n", sizeof("tcc: nonfatal compile error\\n") - 1);\
    tcc_state->nb_errors++;\
}' libtcc.c
      $BB sed -i '/PUB_FUNC void tcc_error(const char \*fmt, ...)/,/^}/c\
PUB_FUNC void tcc_error(const char *fmt, ...)\
{\
    (void)fmt;\
    write(2, "tcc: fatal compile error\\n", sizeof("tcc: fatal compile error\\n") - 1);\
    exit(1);\
}' libtcc.c
      $BB sed -i '/PUB_FUNC void tcc_warning(const char \*fmt, ...)/,/^}/c\
PUB_FUNC void tcc_warning(const char *fmt, ...)\
{\
    (void)fmt;\
    if (!tcc_state->warn_none) write(2, "tcc: compile warning\\n", sizeof("tcc: compile warning\\n") - 1);\
}' libtcc.c

      # Keep the Mes-facing declarations that the source-built predecessor can
      # parse, but use the native-musl x86_64 va_list ABI for every rebuilt
      # compiler object that crosses into native libc's v*printf functions.
      $BB cat > "$WORK/native-abi/stdarg.h" <<'EOF'
#ifndef _STDARG_H
#define _STDARG_H
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __mantle_va_list_struct;
typedef __mantle_va_list_struct va_list[1];
void __va_start(void *, void *);
void *__va_arg(void *, int, int, int);
#define va_start(arguments, last) __va_start((arguments), __builtin_frame_address(0))
#define va_arg(arguments, type) \
    (*(type *)__va_arg((arguments), __builtin_va_arg_types(type), sizeof(type), __alignof__(type)))
#define va_copy(destination, source) (*(destination) = *(source))
#define va_end(arguments)
#endif
EOF

      # Route only source-literal conversion through the bounded parser below;
      # runtime programs continue to use libc's ordinary strto* implementation.
      $BB sed -i 's/strtof(/bootstrap_strtof(/g; s/strtod(/bootstrap_strtod(/g; s/strtold(/bootstrap_strtold(/g' tccpp.c
      # Dependency-list growth crashes after the included file is opened.  It
      # does not affect preprocessing or object semantics, so omit only this
      # metadata edge in the diagnostic compiler and require its restoration
      # before admission.
      $BB sed -i '/\/\* update target deps \*\//,/                    tcc_strdup(buf1));/d' tccpp.c
      # The reduced mmap allocator cannot yet survive TinyCC's full teardown
      # graph.  Each bootstrap invocation compiles one translation unit, so
      # defer successful-process reclamation to the kernel and require the later
      # GCC-built libc/compiler pass to restore tcc_delete before admission.
      $BB sed -i 's/    tcc_delete(s);/    (void)s; \/\* kernel-reclaimed bootstrap process \*\//' tcc.c
      $BB sed -i '/static void parse_number/i\
extern float bootstrap_strtof(const char *, char **);\
extern double bootstrap_strtod(const char *, char **);\
extern long double bootstrap_strtold(const char *, char **);' tccpp.c

      MAIN_OBJECT=""
      LINK_OBJECTS=""
      for compiler_source in tcc.c libtcc.c tccpp.c tccgen.c tccelf.c tccasm.c tccrun.c x86_64-gen.c x86_64-link.c i386-asm.c; do
        object="${compiler_source%.c}.o"
        echo "tcc-native-runtime: compile $compiler_source" >&2
        "$HOST_TCC/bin/tcc" \
          -c -I . -I "$WORK/native-abi" -I "$TCC26/include/mes" \
          -D BOOTSTRAP=1 \
          -D HAVE_BITFIELD=1 \
          -D HAVE_FLOAT=1 \
          -D HAVE_LONG_LONG=1 \
          -D HAVE_SETJMP=1 \
          -D TCC_TARGET_X86_64=1 \
          -D 'CONFIG_TCCDIR="'"$out/lib/tcc"'"' \
          -D 'CONFIG_TCC_CRTPREFIX="'"$MUSL/lib"'"' \
          -D 'CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1"' \
          -D 'CONFIG_TCC_LIBPATHS="'"$MUSL/lib:$out/lib/tcc"'"' \
          -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$out/lib/tcc/include:$MUSL/include"'"' \
          -D 'CONFIG_SYSROOT="/"' \
          -D 'TCC_LIBGCC="'"$MUSL/lib/libc.a"'"' \
          -D 'TCC_LIBTCC1="libtcc1.a"' \
          -D CONFIG_TCCBOOT=1 \
          -D CONFIG_TCC_STATIC=1 \
          -D CONFIG_USE_LIBGCC=1 \
          -D 'TCC_VERSION="0.9.27"' \
          -D ONE_SOURCE=0 \
          -o "$object" "$compiler_source"
        test -s "$object" || { echo "ERROR: native-runtime TinyCC object $object missing" >&2; exit 1; }
        $BB chmod u+rw "$object"
        if [ "$compiler_source" = tcc.c ]; then
          MAIN_OBJECT="$object"
        else
          LINK_OBJECTS="$LINK_OBJECTS $object"
        fi
      done
      test -n "$MAIN_OBJECT" || { echo "ERROR: native-runtime TinyCC main object missing" >&2; exit 1; }
      test -n "$LINK_OBJECTS" || { echo "ERROR: native-runtime TinyCC library objects missing" >&2; exit 1; }

      # TinyCC calls strto* and ldexp while converting source literals.  The
      # reduced libc does not yet provide a trustworthy fractional parser, so
      # this compiler shell uses a bounded decimal/hex parser with no floating
      # source literals of its own.  GCC-built musl must replace this boundary.
      $BB cat > native-runtime-compat.c <<'EOF'
#include <stddef.h>
int errno;
static int bootstrap_digit_value(int character)
{
    if (character >= '0' && character <= '9') return character - '0';
    if (character >= 'a' && character <= 'f') return character - 'a' + 10;
    if (character >= 'A' && character <= 'F') return character - 'A' + 10;
    return -1;
}
static double bootstrap_scale(double value, int base, int exponent)
{
    enum { exponent_limit = 16384 };
    int step_count = 0;
    while (exponent > 0 && step_count < exponent_limit) {
        value *= base;
        exponent--;
        step_count++;
    }
    while (exponent < 0 && step_count < exponent_limit) {
        value /= base;
        exponent++;
        step_count++;
    }
    return value;
}
static double bootstrap_parse_float(const char *text, char **end_pointer)
{
    enum { decimal_base = 10, hexadecimal_base = 16 };
    const char *cursor = text;
    int negative = 0;
    int base = decimal_base;
    int fractional_digits = 0;
    int exponent = 0;
    int exponent_negative = 0;
    double value = 0;
    if (*cursor == '-' || *cursor == '+') negative = *cursor++ == '-';
    if (cursor[0] == '0' && (cursor[1] == 'x' || cursor[1] == 'X')) {
        base = hexadecimal_base;
        cursor += 2;
    }
    while (*cursor) {
        int digit;
        if (*cursor == '.') {
            cursor++;
            while ((digit = bootstrap_digit_value(*cursor)) >= 0 && digit < base) {
                value = value * base + digit;
                fractional_digits++;
                cursor++;
            }
            break;
        }
        digit = bootstrap_digit_value(*cursor);
        if (digit < 0 || digit >= base) break;
        value = value * base + digit;
        cursor++;
    }
    value = bootstrap_scale(value, base, -fractional_digits);
    if (*cursor == 'e' || *cursor == 'E' || *cursor == 'p' || *cursor == 'P') {
        int exponent_base = (*cursor == 'p' || *cursor == 'P') ? 2 : decimal_base;
        cursor++;
        if (*cursor == '-' || *cursor == '+') exponent_negative = *cursor++ == '-';
        while (*cursor >= '0' && *cursor <= '9') {
            exponent = exponent * decimal_base + *cursor - '0';
            cursor++;
        }
        if (exponent_negative) exponent = -exponent;
        value = bootstrap_scale(value, exponent_base, exponent);
    }
    if (end_pointer) *end_pointer = (char *)cursor;
    return negative ? -value : value;
}
float bootstrap_strtof(const char *text, char **end_pointer)
{
    return bootstrap_parse_float(text, end_pointer);
}
double bootstrap_strtod(const char *text, char **end_pointer)
{
    return bootstrap_parse_float(text, end_pointer);
}
long double bootstrap_strtold(const char *text, char **end_pointer)
{
    return (long double)bootstrap_parse_float(text, end_pointer);
}
double ldexp(double value, int exponent)
{
    return bootstrap_scale(value, 2, exponent);
}
EOF
      echo "tcc-native-runtime: compile source-literal adapter" >&2
      "$HOST_TCC/bin/tcc" -c -I "$WORK/native-abi" -I "$TCC26/include/mes" \
        native-runtime-compat.c -o native-runtime-compat.o
      $BB chmod u+rw native-runtime-compat.o

      echo "tcc-native-runtime: build source-authenticated compiler runtime" >&2
      RUNTIME_OBJECTS=""
      for runtime_source in libtcc1.c va_list.c; do
        runtime_object="native-${runtime_source%.c}.o"
        "$HOST_TCC/bin/tcc" -c -I "$TCC27_SRC" -I "$TCC27_SRC/include" \
          "$TCC27_SRC/lib/$runtime_source" -o "$runtime_object"
        test -s "$runtime_object" || { echo "ERROR: native TinyCC runtime object $runtime_object missing" >&2; exit 1; }
        $BB chmod 644 "$runtime_object"
        RUNTIME_OBJECTS="$RUNTIME_OBJECTS $runtime_object"
      done
      for runtime_source in alloca86_64.S alloca86_64-bt.S; do
        runtime_object="native-${runtime_source%.S}.o"
        "$HOST_TCC/bin/tcc" -c -I "$TCC27_SRC" -I "$TCC27_SRC/include" \
          "$TCC27_SRC/lib/$runtime_source" -o "$runtime_object"
        test -s "$runtime_object" || { echo "ERROR: native TinyCC assembly runtime object $runtime_object missing" >&2; exit 1; }
        $BB chmod 644 "$runtime_object"
        RUNTIME_OBJECTS="$RUNTIME_OBJECTS $runtime_object"
      done
      $BB rm -f native-libtcc1.a
      "$BINUTILS/bin/ar" rc native-libtcc1.a $RUNTIME_OBJECTS
      "$BINUTILS/bin/ranlib" native-libtcc1.a
      test -s native-libtcc1.a || { echo "ERROR: source-built native TinyCC runtime archive missing" >&2; exit 1; }

      echo "tcc-native-runtime: link compiler with source-built binutils" >&2
      "$BINUTILS/bin/ld" -static -o tcc-native \
        "$MUSL/lib/crt1.o" "$MAIN_OBJECT" $LINK_OBJECTS \
        native-runtime-compat.o native-libtcc1.a "$MUSL/lib/libc.a"
      $BB chmod 755 tcc-native
      $BB cp tcc-native "$out/bin/tcc"
      $BB cp tcc-native "$out/bin/tcc-0.9.27-musl-native-runtime"
      $BB cp native-libtcc1.a "$out/lib/tcc/libtcc1.a"
      $BB cp -r "$SELFHOST/lib/tcc/include/." "$out/lib/tcc/include/"

      $BB cat > integer-compile-smoke.c <<'EOF'
int value(void) { return 7; }
EOF
      echo "tcc-native-runtime: compile integer smoke" >&2
      "$out/bin/tcc" -c integer-compile-smoke.c -o integer-compile-smoke.o
      test -s integer-compile-smoke.o || { echo "ERROR: native-runtime TinyCC integer smoke missing" >&2; exit 1; }

      $BB cat > include-smoke.h <<'EOF'
#define MANTLE_INCLUDE_SMOKE_VALUE 11
EOF
      $BB cat > include-smoke.c <<'EOF'
#include "include-smoke.h"
int included_value(void) { return MANTLE_INCLUDE_SMOKE_VALUE; }
EOF
      echo "tcc-native-runtime: compile nested-include smoke" >&2
      "$out/bin/tcc" -c include-smoke.c -o include-smoke.o
      test -s include-smoke.o || { echo "ERROR: native-runtime TinyCC include smoke missing" >&2; exit 1; }

      $BB cat > decimal-runtime-smoke.c <<'EOF'
int main(void)
{
    enum { floating_failure = 19 };
    double expected_product = 3.0;
    double input_value = 1.5;
    double product = input_value * 2.0;
    return product == expected_product ? 0 : floating_failure;
}
EOF
      echo "tcc-native-runtime: compile floating smoke" >&2
      "$out/bin/tcc" -c -I"$MUSL/include" decimal-runtime-smoke.c -o decimal-runtime-smoke.o
      $BB chmod u+rw decimal-runtime-smoke.o
      echo "tcc-native-runtime: link floating smoke" >&2
      "$out/bin/tcc" -nostdlib -static -o decimal-runtime-smoke \
        "$MUSL/lib/crt1.o" decimal-runtime-smoke.o \
        "$MUSL/lib/libc.a" "$out/lib/tcc/libtcc1.a"
      $BB chmod 755 decimal-runtime-smoke
      echo "tcc-native-runtime: execute floating smoke" >&2
      ./decimal-runtime-smoke || {
        echo "ERROR: native-runtime TinyCC floating smoke failed" >&2
        exit 1
      }

      $BB cat > malformed-negative.c <<'EOF'
int main(void) { return missing_expression + ; }
EOF
      if "$out/bin/tcc" -c malformed-negative.c -o malformed-negative.o > malformed-negative.log 2>&1; then
        echo "ERROR: malformed TinyCC input unexpectedly compiled" >&2
        exit 1
      fi
      test -s malformed-negative.log || {
        echo "ERROR: malformed TinyCC input emitted no diagnostic" >&2
        exit 1
      }
      $BB grep -q 'tcc: fatal compile error' malformed-negative.log || {
        echo "ERROR: malformed TinyCC input missed the fail-closed diagnostic" >&2
        exit 1
      }

      test -x "$out/bin/tcc"
      test -f "$out/lib/tcc/libtcc1.a"
      test -f "$out/lib/tcc/include/stdarg.h"
    "%,
  ],
  inputs = [stage0, tcc26, host_tcc, selfhost, musl, binutils, tcc27_src],
} | crunch.Derivation
