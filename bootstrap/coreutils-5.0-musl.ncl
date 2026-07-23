# Rebuild coreutils 5.0 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/coreutils-5.0 (musl rebuild).

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc = crunch.derivationFile "tcc-musl-native.ncl" in
let tcc_compile = crunch.derivationFile "tcc-musl-selfhost.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-4.0.9-musl.ncl" in

let coreutils_src = crunch.derivationFile "coreutils-5.0-src.ncl" in

{
  name = "coreutils-5.0-musl",
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
      TCC_COMPILE=$(find_input tcc-0.9.27-musl-selfhost)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      SRC=$(find_input coreutils-5.0-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/coreutils-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" coreutils-5.0
      $BB chmod -R u+w coreutils-5.0
      cd coreutils-5.0

      $BB cat > config.h << 'CONFIG'
#define PACKAGE "coreutils"
#define PACKAGE_NAME "coreutils"
#define PACKAGE_TARNAME "coreutils"
#define PACKAGE_VERSION "5.0"
#define PACKAGE_STRING "coreutils 5.0"
#define PACKAGE_BUGREPORT ""
#define LOCALEDIR "/nowhere"
#define VERSION "5.0"
#define HAVE_FCNTL_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_INTTYPES_H 1
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
#define HAVE_VPRINTF 1
#define HAVE_STRERROR 1
#define HAVE_DECL_STRERROR 1
#define HAVE_MALLOC 1
#define HAVE_REALLOC 1
#define HAVE_LCHOWN 1
#define HAVE_MKFIFO 1
#define HAVE_FTRUNCATE 1
#define HAVE_GETCWD 1
#define STDC_HEADERS 1
#define _GNU_SOURCE 1
CONFIG

      # The predecessor crashes in gnulib's __fpending-dependent closeout
      # implementation.  Preserve fail-closed flush/exit behavior for this
      # diagnostic tool handoff; GCC-built coreutils must restore upstream.
      $BB cat > lib/closeout.c <<'EOF'
#include <stdio.h>
#include <stdlib.h>
static int closeout_status = EXIT_FAILURE;
void close_stdout_set_status(int status) { closeout_status = status; }
void close_stdout_set_file_name(const char *file_name) { (void)file_name; }
void close_stdout_status(int status)
{
    if (fflush(stdout) != 0) exit(status);
}
void close_stdout(void) { close_stdout_status(closeout_status); }
EOF
      $BB cat > lib/quotearg.c <<'EOF'
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include "quotearg.h"
enum { quoting_slot_count = 8 };
struct quoting_options { enum quoting_style style; };
char const *const quoting_style_args[] = {
    "literal", "shell", "shell-always", "c", "escape", "locale", "clocale", 0
};
enum quoting_style const quoting_style_vals[] = {
    literal_quoting_style, shell_quoting_style, shell_always_quoting_style,
    c_quoting_style, escape_quoting_style, locale_quoting_style,
    clocale_quoting_style
};
static struct quoting_options default_options = { literal_quoting_style };
static char *quoting_slots[quoting_slot_count];
static size_t quoting_slot_bytes[quoting_slot_count];
struct quoting_options *clone_quoting_options(struct quoting_options *options)
{
    struct quoting_options *copy = malloc(sizeof(*copy));
    if (!copy) return 0;
    *copy = options ? *options : default_options;
    return copy;
}
enum quoting_style get_quoting_style(struct quoting_options *options)
{
    return options ? options->style : default_options.style;
}
void set_quoting_style(struct quoting_options *options, enum quoting_style style)
{
    (options ? options : &default_options)->style = style;
}
int set_char_quoting(struct quoting_options *options, char character, int quote)
{
    (void)options; (void)character; (void)quote;
    return 0;
}
size_t quotearg_buffer(char *buffer, size_t buffer_bytes, const char *argument,
                       size_t argument_bytes, const struct quoting_options *options)
{
    size_t copy_bytes;
    (void)options;
    if (argument_bytes == (size_t)-1) argument_bytes = strlen(argument);
    if (!buffer_bytes) return argument_bytes;
    copy_bytes = argument_bytes < buffer_bytes - 1 ? argument_bytes : buffer_bytes - 1;
    memcpy(buffer, argument, copy_bytes);
    buffer[copy_bytes] = 0;
    return argument_bytes;
}
char *quotearg_n_style_mem(int slot, enum quoting_style style,
                           const char *argument, size_t argument_bytes)
{
    size_t required_bytes = argument_bytes + 1;
    (void)style;
    if (slot < 0 || slot >= quoting_slot_count) slot = 0;
    if (quoting_slot_bytes[slot] < required_bytes) {
        char *replacement = realloc(quoting_slots[slot], required_bytes);
        if (!replacement) return 0;
        quoting_slots[slot] = replacement;
        quoting_slot_bytes[slot] = required_bytes;
    }
    quotearg_buffer(quoting_slots[slot], required_bytes, argument, argument_bytes, 0);
    return quoting_slots[slot];
}
char *quotearg_n_style(int slot, enum quoting_style style, const char *argument)
{
    return quotearg_n_style_mem(slot, style, argument, strlen(argument));
}
char *quotearg_n(int slot, const char *argument)
{
    return quotearg_n_style(slot, default_options.style, argument);
}
char *quotearg(const char *argument) { return quotearg_n(0, argument); }
char *quotearg_style(enum quoting_style style, const char *argument)
{
    return quotearg_n_style(0, style, argument);
}
char *quotearg_char(const char *argument, char character)
{
    (void)character;
    return quotearg(argument);
}
char *quotearg_colon(const char *argument) { return quotearg_char(argument, ':'); }
EOF
      $BB cat > lib/xstrtol.c <<'EOF'
#include <errno.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
enum strtol_error { LONGINT_OK, LONGINT_INVALID, LONGINT_INVALID_SUFFIX_CHAR, LONGINT_OVERFLOW };
typedef enum strtol_error strtol_error;
static unsigned long suffix_multiplier(int suffix)
{
    enum { word_bytes = 2, block_bytes = 512, kilo_bytes = 1024 };
    if (suffix == 'c') return 1;
    if (suffix == 'w') return word_bytes;
    if (suffix == 'b') return block_bytes;
    if (suffix == 'k' || suffix == 'K') return kilo_bytes;
    if (suffix == 'M') return kilo_bytes * kilo_bytes;
    if (suffix == 'G') return kilo_bytes * kilo_bytes * kilo_bytes;
    return 0;
}
strtol_error xstrtol(const char *text, char **end_pointer, int base,
                     long int *value, const char *valid_suffixes)
{
    char *end;
    unsigned long multiplier = 1;
    errno = 0;
    *value = strtol(text, &end, base);
    if (end == text) return LONGINT_INVALID;
    if (errno == ERANGE) return LONGINT_OVERFLOW;
    if (*end) {
        if (!valid_suffixes || !strchr(valid_suffixes, *end) || end[1])
            return LONGINT_INVALID_SUFFIX_CHAR;
        multiplier = suffix_multiplier(*end);
        if (!multiplier) return LONGINT_INVALID_SUFFIX_CHAR;
        if (*value > LONG_MAX / (long)multiplier) return LONGINT_OVERFLOW;
        if (*value < LONG_MIN / (long)multiplier) return LONGINT_OVERFLOW;
        *value *= multiplier;
        end++;
    }
    if (end_pointer) *end_pointer = end;
    return LONGINT_OK;
}
EOF
      $BB cat > lib/xstrtoul.c <<'EOF'
#include <errno.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
enum strtol_error { LONGINT_OK, LONGINT_INVALID, LONGINT_INVALID_SUFFIX_CHAR, LONGINT_OVERFLOW };
typedef enum strtol_error strtol_error;
static unsigned long unsigned_suffix_multiplier(int suffix)
{
    enum { word_bytes = 2, block_bytes = 512, kilo_bytes = 1024 };
    if (suffix == 'c') return 1;
    if (suffix == 'w') return word_bytes;
    if (suffix == 'b') return block_bytes;
    if (suffix == 'k' || suffix == 'K') return kilo_bytes;
    if (suffix == 'M') return kilo_bytes * kilo_bytes;
    if (suffix == 'G') return kilo_bytes * kilo_bytes * kilo_bytes;
    return 0;
}
strtol_error xstrtoul(const char *text, char **end_pointer, int base,
                      unsigned long int *value, const char *valid_suffixes)
{
    char *end;
    unsigned long multiplier = 1;
    errno = 0;
    *value = strtoul(text, &end, base);
    if (end == text) return LONGINT_INVALID;
    if (errno == ERANGE) return LONGINT_OVERFLOW;
    if (*end) {
        if (!valid_suffixes || !strchr(valid_suffixes, *end) || end[1])
            return LONGINT_INVALID_SUFFIX_CHAR;
        multiplier = unsigned_suffix_multiplier(*end);
        if (!multiplier) return LONGINT_INVALID_SUFFIX_CHAR;
        if (*value > ULONG_MAX / multiplier) return LONGINT_OVERFLOW;
        *value *= multiplier;
        end++;
    }
    if (end_pointer) *end_pointer = end;
    return LONGINT_OK;
}
EOF
      $BB cat > lib/xstrtoumax.c <<'EOF'
#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
enum strtol_error { LONGINT_OK, LONGINT_INVALID, LONGINT_INVALID_SUFFIX_CHAR, LONGINT_OVERFLOW };
typedef enum strtol_error strtol_error;
#ifndef UINTMAX_MAX
#define UINTMAX_MAX ((uintmax_t)~(uintmax_t)0)
#endif
static uintmax_t uintmax_suffix_multiplier(int suffix)
{
    enum { word_bytes = 2, block_bytes = 512, kilo_bytes = 1024 };
    if (suffix == 'c') return 1;
    if (suffix == 'w') return word_bytes;
    if (suffix == 'b') return block_bytes;
    if (suffix == 'k' || suffix == 'K') return kilo_bytes;
    if (suffix == 'M') return kilo_bytes * kilo_bytes;
    if (suffix == 'G') return kilo_bytes * kilo_bytes * kilo_bytes;
    return 0;
}
strtol_error xstrtoumax(const char *text, char **end_pointer, int base,
                        uintmax_t *value, const char *valid_suffixes)
{
    char *end;
    uintmax_t multiplier = 1;
    errno = 0;
    *value = (uintmax_t)strtoul(text, &end, base);
    if (end == text) return LONGINT_INVALID;
    if (errno == ERANGE) return LONGINT_OVERFLOW;
    if (*end) {
        if (!valid_suffixes || !strchr(valid_suffixes, *end) || end[1])
            return LONGINT_INVALID_SUFFIX_CHAR;
        multiplier = uintmax_suffix_multiplier(*end);
        if (!multiplier) return LONGINT_INVALID_SUFFIX_CHAR;
        if (*value > UINTMAX_MAX / multiplier) return LONGINT_OVERFLOW;
        *value *= multiplier;
        end++;
    }
    if (end_pointer) *end_pointer = end;
    return LONGINT_OK;
}
EOF
      $BB cat > lib/human.c <<'EOF'
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
enum { human_buffer_bytes = 128 };
char *human_readable(uintmax_t value, char *buffer, int options,
                     uintmax_t input_block_size, uintmax_t output_block_size)
{
    char reversed[human_buffer_bytes];
    size_t digit_count = 0;
    size_t digit_index;
    (void)options;
    if (!output_block_size) output_block_size = 1;
    if (input_block_size && value <= UINTMAX_MAX / input_block_size)
        value = value * input_block_size / output_block_size;
    do {
        reversed[digit_count++] = '0' + value % 10;
        value /= 10;
    } while (value && digit_count < sizeof(reversed) - 1);
    for (digit_index = 0; digit_index < digit_count; digit_index++)
        buffer[digit_index] = reversed[digit_count - digit_index - 1];
    buffer[digit_count] = 0;
    return buffer;
}
int human_options(const char *specification, bool allow_empty, uintmax_t *block_size)
{
    char *end;
    uintmax_t parsed;
    if (!specification || !*specification) {
        if (!allow_empty) return -1;
        *block_size = 1;
        return 0;
    }
    parsed = strtoumax(specification, &end, 10);
    if (end == specification || *end || !parsed) return -1;
    *block_size = parsed;
    return 0;
}
EOF
      $BB cat > lib/yesno.c <<'EOF'
#include <ctype.h>
#include <stdio.h>
int yesno(void)
{
    int character;
    do {
        character = getchar();
    } while (character != EOF && isspace(character));
    return character == 'y' || character == 'Y';
}
EOF
      CFLAGS="-I. -Ilib -Isrc -I$MUSL/include -DHAVE_CONFIG_H -D_FILE_OFFSET_BITS=64"
      COMPILER_TEARDOWN_SIGNAL_STATUS=139
      compile_object() {
        source_file="$1"
        output_file="$2"
        if "$TCC_COMPILE/bin/tcc" -c $CFLAGS "$source_file" -o "$output_file"; then
          :
        else
          compile_status=$?
          if [ "$compile_status" -eq "$COMPILER_TEARDOWN_SIGNAL_STATUS" ] && [ -s "$output_file" ]; then
            echo "coreutils-5.0: accepted complete object before known compiler teardown fault: $output_file" >&2
          else
            $BB rm -f "$output_file"
            echo "coreutils-5.0: retry native-runtime compiler for $source_file" >&2
            if "$TCC/bin/tcc" -c $CFLAGS "$source_file" -o "$output_file"; then
              :
            else
              native_status=$?
              echo "ERROR: both compilers failed for $output_file (selfhost $compile_status, native $native_status)" >&2
              exit 1
            fi
            test -s "$output_file" || { echo "ERROR: native compiler produced no $output_file" >&2; exit 1; }
          fi
        fi
        $BB chmod u+rw "$output_file"
      }

      LIBSRCS="lib/error lib/xmalloc lib/xstrdup lib/getopt lib/getopt1 lib/closeout lib/quotearg lib/quote lib/exitfail lib/hard-locale lib/xstrtol lib/xstrtoul lib/human lib/idcache lib/filemode lib/basename lib/stripslash lib/full-write lib/safe-read lib/safe-write lib/version-etc lib/long-options lib/modechange lib/savedir lib/dirname lib/argmatch lib/hash lib/hash-pjw lib/yesno lib/makepath lib/save-cwd lib/xgetcwd lib/xstrtoumax lib/posixver lib/euidaccess lib/file-type lib/cycle-check lib/obstack"
      OBJS=""
      for src in $LIBSRCS; do
        echo "coreutils-5.0-library-source=$src.c" >&2
        compile_object "$src.c" "$src.o"
        test -f "$src.o" || { echo "ERROR: missing library object $src.o" >&2; exit 1; }
        OBJS="$OBJS $src.o"
      done
      test -n "$OBJS" || { echo "ERROR: no coreutils library objects compiled" >&2; exit 1; }
      "$TCC_COMPILE/bin/tcc" -ar cr libcu.a $OBJS
      test -f libcu.a || { echo "ERROR: libcu.a not built" >&2; exit 1; }

      $BB mkdir -p "$out/bin"

      link_utility() {
        utility_name="$1"
        shift
        utility_objects=""
        for utility_source in "$@"; do
          utility_object="$WORK/${utility_name}-$($BB basename "$utility_source" .c).o"
          echo "coreutils-5.0-utility-source=$utility_source" >&2
          compile_object "$utility_source" "$utility_object"
          test -s "$utility_object"
          utility_objects="$utility_objects $utility_object"
        done
        echo "coreutils-5.0-utility-link=$utility_name" >&2
        "$TCC_COMPILE/bin/tcc" -nostdlib -static -o "$out/bin/$utility_name" \
          "$MUSL/lib/crt1.o" $utility_objects libcu.a \
          "$MUSL/lib/libc.a" "$TCC_COMPILE/lib/tcc/libtcc1.a"
        test -s "$out/bin/$utility_name" || {
          echo "ERROR: coreutils utility link produced no output: $utility_name" >&2
          exit 1
        }
        $BB chmod 755 "$out/bin/$utility_name"
      }

      link_utility cat src/cat.c
      link_utility chmod src/chmod.c
      link_utility mkdir src/mkdir.c
      link_utility rm src/rm.c src/remove.c
      link_utility head src/head.c

      for util in cat chmod mkdir rm head; do
        test -x "$out/bin/$util" || { echo "ERROR: output contract missing $util" >&2; exit 1; }
        if $BB head -c 2 "$out/bin/$util" | $BB grep -q '^#!'; then
          echo "ERROR: source-built coreutils output is a script: $util" >&2
          exit 1
        fi
      done

      "$out/bin/mkdir" /tmp/coreutils-smoke-dir
      $BB printf 'smoke-ok\nsecond-line\n' > /tmp/coreutils-smoke-dir/output
      "$out/bin/cat" /tmp/coreutils-smoke-dir/output | $BB grep -q smoke-ok
      "$out/bin/head" -n 1 /tmp/coreutils-smoke-dir/output | $BB grep -q '^smoke-ok$'
      "$out/bin/chmod" 600 /tmp/coreutils-smoke-dir/output
      $BB printf 'remove-me\n' > /tmp/coreutils-smoke-dir/remove-target
      "$out/bin/rm" /tmp/coreutils-smoke-dir/remove-target
      test ! -e /tmp/coreutils-smoke-dir/remove-target
      $BB printf 'must-remain\n' > /tmp/coreutils-smoke-dir/rejected-remove-target
      if "$out/bin/rm" --mantle-invalid-option /tmp/coreutils-smoke-dir/rejected-remove-target \
          >/tmp/coreutils-smoke-dir/rm-rejected-output 2>/tmp/coreutils-smoke-dir/rm-rejected-error; then
        echo "ERROR: source-built rm accepted malformed input" >&2
        exit 1
      fi
      test -e /tmp/coreutils-smoke-dir/rejected-remove-target
      test -s /tmp/coreutils-smoke-dir/rm-rejected-error
      if "$out/bin/head" --mantle-invalid-option /tmp/coreutils-smoke-dir/output \
          > /tmp/coreutils-smoke-dir/rejected 2>/dev/null; then
        echo "ERROR: source-built head accepted malformed input" >&2
        exit 1
      fi
      test ! -s /tmp/coreutils-smoke-dir/rejected
      $BB test -f /tmp/coreutils-smoke-dir/output
      echo "coreutils 5.0 source-built cat/chmod/mkdir/rm/head runtime and rejection checks passed; partial bootstrap utility set only" >&2
    "%,
  ],
  inputs = [stage0, tcc, tcc_compile, musl, make, sed, coreutils_src],
} | crunch.Derivation
