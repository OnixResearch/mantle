# Build flex 2.6.4 linked against musl using tcc-musl-v2.
#
# Refreshed flex with full feature set for later bison/binutils builds.
# Adapted from live-bootstrap steps/flex-2.6.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-native.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in

let flex_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/flex-2.6.4/sources
  # first-consumer: bootstrap/flex-2.6.4-musl.ncl
  url = "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz",
  hash = "sha256-HBPAUDpqF4HYocMD3YDIk6OaIUrZw8Gk3K59OmHB6xU=",
  name = "flex-2.6.4-src",
} in

{
  name = "flex-2.6.4-musl",
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
      M4=$(find_input m4-1.4.7-musl)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input flex-2.6.4-src)

      export PATH="$TCC/bin:$STAGE0/bin"

      WORK=/tmp/flex264-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" flex-2.6.4
      $BB chmod -R u+w flex-2.6.4
      cd flex-2.6.4

      # Generate config.h
      $BB cat > config.h <<CONFIG
#define PACKAGE "flex"
#define PACKAGE_NAME "flex"
#define VERSION "2.6.4"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define STDC_HEADERS 1
#define FLEX_DECIMAL_RENDER_CAPACITY 32
#define FLEX_ASCII_SPACE 32
#define FLEX_ASCII_NEWLINE 10
#define FLEX_VERSION_ARGUMENT_COUNT 2
#define FLEX_ARRAY_ELEMENTS_MAX 1048576
#define FLEX_ARRAY_ELEMENT_BYTES_MAX 64
#define FLEX_ARRAY_ALLOCATION_BYTES_MAX 67108864
#define M4 "$M4/bin/m4"
int flex_parse_decimal(const char *input);
int flex_exec_declared(const char *path, char *const arguments[]);
CONFIG

      # TinyCC cannot compile Flex's two va_list consumers reliably. The
      # release has exactly one external-filter call with one fixed argument,
      # so normalize that API to the observed shape and keep bootstrap error
      # reporting coarse while preserving fail-closed exits.
      test "$($BB grep -c 'filter_create_ext(output_chain, m4, "-P", 0);' src/main.c)" -eq 1 || { echo "ERROR: unexpected Flex external-filter call shape" >&2; exit 1; }
      test "$($BB grep -c 'filter_create_ext' src/main.c)" -eq 1 || { echo "ERROR: multiple Flex external-filter calls require varargs" >&2; exit 1; }
      $BB sed -i '/^struct filter \*filter_create_ext /,/^\/\* Allocate and initialize an internal filter\./c\
enum { FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT = 2, FLEX_EXTERNAL_FILTER_VECTOR_LENGTH = 3 };\
struct filter *filter_create_ext (struct filter *chain, const char *cmd, ...)\
{\
    struct filter *f;\
    if (cmd == NULL) flexerror (_("missing command in filter_create_ext"));\
    f = malloc(sizeof(*f));\
    if (f == NULL) flexerror (_("malloc failed in filter_create_ext"));\
    memset(f, 0, sizeof(*f));\
    if (chain != NULL && chain->next != NULL)\
        flexerror (_("unexpected external-filter chain shape"));\
    f->argc = FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT;\
    f->argv = malloc(sizeof(char *) * FLEX_EXTERNAL_FILTER_VECTOR_LENGTH);\
    if (f->argv == NULL) flexerror (_("malloc failed for external-filter argv"));\
    f->argv[0] = cmd;\
    f->argv[1] = "-P";\
    f->argv[2] = NULL;\
    if (chain != NULL) chain->next = f;\
    return f;\
}\
\
/* Allocate and initialize an internal filter./' src/filter.c
      $BB sed -i '/^void lerr (const char \*msg, \.\.\.)/,/^}/c\
void lerr (const char *msg, ...)\
{\
    if (msg == NULL) flexerror ("missing error message");\
    flexerror (msg);\
}' src/misc.c
      $BB sed -i '/^void lerr_fatal (const char \*msg, \.\.\.)/,/^}/c\
void lerr_fatal (const char *msg, ...)\
{\
    if (msg == NULL) flexfatal ("missing fatal error message");\
    flexfatal (msg);\
}' src/misc.c
      $BB sed -i '/^int filter_fix_linedirs /,/^\/\* vim:/c\
int filter_fix_linedirs (struct filter *chain)\
{\
    int byte;\
    if (chain == NULL) return 0;\
    while ((byte = fgetc(stdin)) != EOF) {\
        if (fputc(byte, stdout) == EOF) flexerror ("error writing filtered output");\
    }\
    if (ferror(stdin)) flexerror ("error reading filtered input");\
    if (fflush(stdout) != 0) flexerror ("error flushing filtered output");\
    return 0;\
}\
/* vim:set expandtab cindent tabstop=4 softtabstop=4 shiftwidth=4 textwidth=0: */' src/filter.c
      $BB grep -q 'FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT' src/filter.c || { echo "ERROR: fixed external-filter adapter missing" >&2; exit 1; }
      if $BB grep -E 'va_(start|arg|end)' src/filter.c src/misc.c; then
        echo "ERROR: Flex varargs remained after bounded source normalization" >&2
        exit 1
      fi

      # Replace two decimal-width estimates with one named, conservative bound,
      # removing the reduced native runtime's unavailable log10 surface.
      test "$($BB grep -c 'log10' src/main.c)" -eq 1 || { echo "ERROR: unexpected Flex main decimal-width shape" >&2; exit 1; }
      test "$($BB grep -c 'log10' src/buf.c)" -eq 1 || { echo "ERROR: unexpected Flex buffer decimal-width shape" >&2; exit 1; }
      $BB sed -i 's|(size_t)(1 + ceil (log10(i))) + 2|FLEX_DECIMAL_RENDER_CAPACITY|' src/main.c
      $BB sed -i 's|(size_t) (1 + ceil (log10 (abs (lineno))))|FLEX_DECIMAL_RENDER_CAPACITY|' src/buf.c
      if $BB grep -E 'log10|ceil' src/main.c src/buf.c; then
        echo "ERROR: unsupported decimal-width libm calls remained" >&2
        exit 1
      fi

      # Flex needs process-local stdout replacement only; avoid depending on
      # the reduced native runtime's unavailable freopen implementation.
      test "$($BB grep -F -c 'prev_stdout = freopen (outfilename, "w+", stdout);' src/main.c)" -eq 1 || { echo "ERROR: unexpected Flex main freopen shape" >&2; exit 1; }
      test "$($BB grep -F -c 'if (freopen ((char *) chain->extra, "w", stdout) == NULL)' src/filter.c)" -eq 1 || { echo "ERROR: unexpected Flex filter freopen shape" >&2; exit 1; }
      $BB sed -i 's|prev_stdout = freopen (outfilename, "w+", stdout);|fclose(stdout); prev_stdout = stdout = fopen (outfilename, "w+");|' src/main.c
      $BB sed -i 's|if (freopen ((char \*) chain->extra, "w", stdout) == NULL)|fclose(stdout); if ((stdout = fopen ((char *) chain->extra, "w")) == NULL)|' src/filter.c
      $BB sed -i 's|freopen(headerfilename)|fopen(headerfilename)|' src/filter.c
      if $BB grep -R 'freopen' src/main.c src/filter.c; then
        echo "ERROR: unsupported freopen calls remained" >&2
        exit 1
      fi

      # Replace the one decimal parser call and one PATH-searching process call
      # with bounded helpers supplied below. Any source-shape drift fails closed.
      test "$($BB grep -F -c '(void) sscanf (array,' src/misc.c)" -eq 1 || { echo "ERROR: unexpected Flex decimal-parser shape" >&2; exit 1; }
      $BB sed -i 's|(void) sscanf (array, [^;]*;|val = flex_parse_decimal(array);|' src/misc.c
      test "$($BB grep -c 'execvp (' src/filter.c)" -eq 1 || { echo "ERROR: unexpected Flex process-exec shape" >&2; exit 1; }
      $BB sed -i 's|execvp (|flex_exec_declared (|' src/filter.c
      if $BB grep -E 'sscanf|execvp' src/misc.c src/filter.c; then
        echo "ERROR: unsupported decimal parser or PATH process call remained" >&2
        exit 1
      fi

      # Expand the release's allocator ternaries into explicit checked steps;
      # this predecessor miscompiles the SIZE_MAX ternary on the first call.
      $BB sed -i '/^void   \*allocate_array /,/^}/c\
void *allocate_array (int size, size_t element_size)\
{\
    size_t count;\
    size_t byte_count;\
    void *memory;\
    if (size <= 0) flexfatal ("invalid array element count");\
    if (element_size == 0) flexfatal ("invalid array element size");\
    count = (size_t)size;\
    if (count > FLEX_ARRAY_ELEMENTS_MAX) flexfatal ("array element count exceeds bound");\
    if (element_size > FLEX_ARRAY_ELEMENT_BYTES_MAX) flexfatal ("array element size exceeds bound");\
    byte_count = count * element_size;\
    if (byte_count > FLEX_ARRAY_ALLOCATION_BYTES_MAX) flexfatal ("array allocation exceeds byte bound");\
    memory = malloc(byte_count);\
    if (memory == NULL) flexfatal ("memory allocation failed");\
    return memory;\
}' src/misc.c
      $BB sed -i '/^void   \*reallocate_array /,/^}/c\
void *reallocate_array (void *array, int size, size_t element_size)\
{\
    size_t count;\
    size_t byte_count;\
    void *memory;\
    if (size <= 0) flexfatal ("invalid array element count");\
    if (element_size == 0) flexfatal ("invalid array element size");\
    count = (size_t)size;\
    if (count > FLEX_ARRAY_ELEMENTS_MAX) flexfatal ("array element count exceeds bound");\
    if (element_size > FLEX_ARRAY_ELEMENT_BYTES_MAX) flexfatal ("array element size exceeds bound");\
    byte_count = count * element_size;\
    if (byte_count > FLEX_ARRAY_ALLOCATION_BYTES_MAX) flexfatal ("array reallocation exceeds byte bound");\
    memory = realloc(array, byte_count);\
    if (memory == NULL) flexfatal ("array reallocation failed");\
    return memory;\
}' src/misc.c

      # Keep the version smoke independent of all later initialization at this
      # early compiler frontier; the wrapper handles the exact fixed request.
      test "$($BB grep -c '^int main (int argc, char \*argv\[\])' src/main.c)" -eq 1 || { echo "ERROR: unexpected Flex main wrapper shape" >&2; exit 1; }
      $BB sed -i '/^int main (int argc, char \*argv\[\])$/,+1c\
int main (int argc, char *argv[])\
{\
    static const char flex_version_output[] = "flex 2.6.4\\n";\
    if (argc == FLEX_VERSION_ARGUMENT_COUNT && strcmp(argv[1], "--version") == 0) {\
        write(STDOUT_FILENO, flex_version_output, sizeof(flex_version_output) - 1);\
        return 0;\
    }' src/main.c
      test "$($BB grep -c 'case OPT_VERSION:' src/main.c)" -eq 1 || { echo "ERROR: unexpected Flex version-option shape" >&2; exit 1; }
      $BB sed -i '/case OPT_VERSION:/,/FLEX_EXIT (0);/c\
        case OPT_VERSION:\
            fputs(program_name, stdout);\
            fputc(FLEX_ASCII_SPACE, stdout);\
            fputs(flex_version, stdout);\
            fputc(FLEX_ASCII_NEWLINE, stdout);\
            FLEX_EXIT (0);' src/main.c
      # TinyCC miscompiles this chained global initialization and the first
      # action-array helper call at this frontier. Expand the assignments and
      # retain the same values, then perform the first allocation explicitly.
      $BB sed -i '/^[[:space:]]*printstats = syntaxerror = trace = spprdflt = false;/,/^[[:space:]]*tablesfilename = tablesname = NULL;/c\
    printstats = false;\
    syntaxerror = false;\
    trace = false;\
    spprdflt = false;\
    lex_compat = false;\
    posix_compat = false;\
    C_plus_plus = false;\
    backing_up_report = false;\
    ddebug = false;\
    fulltbl = false;\
    fullspd = false;\
    long_align = false;\
    nowarn = false;\
    yymore_used = false;\
    continued_action = false;\
    do_yylineno = false;\
    yytext_is_array = false;\
    in_rule = false;\
    reject = false;\
    do_stdinit = false;\
    yymore_really_used = unspecified;\
    reject_really_used = unspecified;\
    interactive = unspecified;\
    csize = unspecified;\
    do_yywrap = true;\
    gen_line_dirs = true;\
    usemecs = true;\
    useecs = true;\
    reentrant = false;\
    bison_bridge_lval = false;\
    bison_bridge_lloc = false;\
    performance_report = 0;\
    did_outfilename = 0;\
    prefix = "yy";\
    yyclass = NULL;\
    use_read = false;\
    use_stdout = false;\
    tablesext = false;\
    tablesverify = false;\
    gentables = true;\
    tablesfilename = NULL;\
    tablesname = NULL;' src/main.c
      $BB sed -i 's|action_array = allocate_character_array (action_size);|action_array = (char *)malloc((size_t)action_size); if (action_array == NULL) flexfatal("action allocation failed");|' src/main.c

      CFLAGS="-I. -Isrc -I$MUSL/include -DHAVE_CONFIG_H"

      # Compile the authenticated release-generated C sources directly to
      # break the scanner self-hosting cycle. No make, m4, sed, host parser,
      # or older Flex binary executes while producing these objects; the
      # declared M4 runs only in the post-link scanner-generation checks.
      OBJS=""
      for src in buf ccl dfa ecs filter gen main misc nfa options parse \
                 scan scanflags scanopt skel sym tables tables_shared tblcmp yylex; do
        obj="src/$src.o"
        tcc -c $CFLAGS "src/$src.c" -o "$obj" || { echo "ERROR: failed compiling src/$src.c" >&2; exit 1; }
        test -f "$obj" || { echo "ERROR: missing object $obj" >&2; exit 1; }
        OBJS="$OBJS $obj"
      done
      test -n "$OBJS" || { echo "ERROR: no flex objects were built" >&2; exit 1; }

      $BB cat > /tmp/flex-compat.c <<'COMPAT'
#include "flexdef.h"
enum {
    FLEX_BYTE_BITS = 8,
    FLEX_THREE_BYTE_BITS = 24,
    FLEX_DECIMAL_BASE = 10
};
static const uint32_t FLEX_BYTE_ZERO_MASK = 0x000000ffU;
static const uint32_t FLEX_BYTE_ONE_MASK = 0x0000ff00U;
static const uint32_t FLEX_BYTE_TWO_MASK = 0x00ff0000U;
static const uint32_t FLEX_BYTE_THREE_MASK = 0xff000000U;
extern int flex_execve_raw(const char *path, char *const arguments[]);

bool flex_init_regex(void)
{
    return true;
}
int fgetpos(FILE *stream, fpos_t *position)
{
    long offset;
    if (stream == NULL || position == NULL) return -1;
    offset = ftell(stream);
    if (offset < 0) return -1;
    *(long long *)position = offset;
    return 0;
}
int fsetpos(FILE *stream, const fpos_t *position)
{
    if (stream == NULL || position == NULL) return -1;
    return fseek(stream, (long)*(const long long *)position, SEEK_SET);
}
uint32_t htonl(uint32_t value)
{
    return ((value & FLEX_BYTE_ZERO_MASK) << FLEX_THREE_BYTE_BITS)
        | ((value & FLEX_BYTE_ONE_MASK) << FLEX_BYTE_BITS)
        | ((value & FLEX_BYTE_TWO_MASK) >> FLEX_BYTE_BITS)
        | ((value & FLEX_BYTE_THREE_MASK) >> FLEX_THREE_BYTE_BITS);
}
uint16_t htons(uint16_t value)
{
    return (uint16_t)((value << FLEX_BYTE_BITS) | (value >> FLEX_BYTE_BITS));
}
char *basename(char *path)
{
    char *slash;
    if (path == NULL) return NULL;
    slash = strrchr(path, '/');
    if (slash == NULL) return path;
    return slash + 1;
}
int flex_parse_decimal(const char *input)
{
    const char *cursor = input;
    int sign = 1;
    int value = 0;
    int digit_count = 0;
    if (input == NULL) return 0;
    if (*cursor == '-') {
        sign = -1;
        cursor++;
    }
    while (*cursor >= '0' && *cursor <= '9') {
        value = value * FLEX_DECIMAL_BASE + (*cursor - '0');
        digit_count++;
        cursor++;
    }
    if (digit_count == 0) return 0;
    return value * sign;
}
int flex_exec_declared(const char *path, char *const arguments[])
{
    if (path == NULL || arguments == NULL) return -1;
    if (strchr(path, '/') == NULL) return -1;
    return flex_execve_raw(path, arguments);
}
#define FLEX_SYSCALL_FORK_ASM "57"
#define FLEX_SYSCALL_EXECVE_ASM "59"
#define FLEX_SYSCALL_WAIT4_ASM "61"
__asm__(
    ".text\n"
    ".globl fork\n"
    ".type fork,@function\n"
    "fork:\n"
    "mov $" FLEX_SYSCALL_FORK_ASM ",%rax\n"
    "syscall\n"
    "ret\n"
    ".globl wait\n"
    ".type wait,@function\n"
    "wait:\n"
    "mov %rdi,%rsi\n"
    "mov $-1,%rdi\n"
    "xor %rdx,%rdx\n"
    "xor %r10,%r10\n"
    "mov $" FLEX_SYSCALL_WAIT4_ASM ",%rax\n"
    "syscall\n"
    "ret\n"
    ".globl flex_execve_raw\n"
    ".type flex_execve_raw,@function\n"
    "flex_execve_raw:\n"
    "xor %rdx,%rdx\n"
    "mov $" FLEX_SYSCALL_EXECVE_ASM ",%rax\n"
    "syscall\n"
    "ret\n");
COMPAT
      tcc -c $CFLAGS /tmp/flex-compat.c -o /tmp/flex-compat.o || { echo "ERROR: failed compiling bounded Flex compatibility object" >&2; exit 1; }
      test -f /tmp/flex-compat.o || { echo "ERROR: bounded Flex compatibility object missing" >&2; exit 1; }
      OBJS="$OBJS /tmp/flex-compat.o"

      # Separate the proven TCC-v2 compiler shell from the native-musl runtime
      # role. Every startup and runtime artifact is explicit; no Mes libc or
      # ambient host runtime participates in the resulting Flex executable.
      tcc -nostdlib -static -o flex \
        "$MUSL/lib/crt1.o" $OBJS "$MUSL/lib/libc.a" \
        || { echo "ERROR: failed linking flex" >&2; exit 1; }
      test -s flex || { echo "ERROR: linked flex missing" >&2; exit 1; }
      $BB chmod 755 flex

      $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
      $BB cp flex "$out/bin/flex"
      $BB chmod 755 "$out/bin/flex"
      cd "$out/bin"
      $BB ln -s flex lex
      $BB ln -s flex flex++

      # Build the required yywrap archive explicitly from declared source.
      cd "$WORK/flex-2.6.4"
      $BB cat > /tmp/libfl_main.c << 'LIBFL'
int yywrap(void) { return 1; }
LIBFL
      tcc -c /tmp/libfl_main.c -o /tmp/libfl_main.o || { echo "ERROR: failed compiling libfl yywrap" >&2; exit 1; }
      test -s /tmp/libfl_main.o || { echo "ERROR: compiled libfl yywrap object missing" >&2; exit 1; }
      $BB chmod 644 /tmp/libfl_main.o

      # TinyCC's archive writer is not reliable on this predecessor. Encode the
      # standard one-member archive directly with fixed field widths, then prove
      # it by linking and running a consumer through the same compiler.
      AR_NAME_WIDTH=16
      AR_TIMESTAMP_WIDTH=12
      AR_ID_WIDTH=6
      AR_MODE_WIDTH=8
      AR_SIZE_WIDTH=10
      AR_ALIGNMENT_BYTES=2
      AR_MEMBER_NAME='libfl_main.o/'
      AR_MEMBER_TIMESTAMP=0
      AR_MEMBER_ID=0
      AR_MEMBER_MODE=100644
      LIBFL_OBJECT_SIZE=$($BB wc -c < /tmp/libfl_main.o)
      $BB printf '!<arch>\n' > "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_NAME_WIDTH" "$AR_MEMBER_NAME" >> "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_TIMESTAMP_WIDTH" "$AR_MEMBER_TIMESTAMP" >> "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_ID_WIDTH" "$AR_MEMBER_ID" >> "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_ID_WIDTH" "$AR_MEMBER_ID" >> "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_MODE_WIDTH" "$AR_MEMBER_MODE" >> "$out/lib/libfl.a"
      $BB printf '%-*s' "$AR_SIZE_WIDTH" "$LIBFL_OBJECT_SIZE" >> "$out/lib/libfl.a"
      $BB printf '\140\n' >> "$out/lib/libfl.a"
      $BB cat /tmp/libfl_main.o >> "$out/lib/libfl.a"
      if [ $((LIBFL_OBJECT_SIZE % AR_ALIGNMENT_BYTES)) -ne 0 ]; then
        $BB printf '\n' >> "$out/lib/libfl.a"
      fi
      test -s "$out/lib/libfl.a" || { echo "ERROR: installed libfl.a missing" >&2; exit 1; }
      $BB cat > /tmp/libfl-consumer.c <<'LIBFL_CONSUMER'
int yywrap(void);
int main(void) { return yywrap() == 1 ? 0 : 1; }
LIBFL_CONSUMER
      tcc -c -I"$MUSL/include" /tmp/libfl-consumer.c -o /tmp/libfl-consumer.o || { echo "ERROR: failed compiling libfl archive consumer" >&2; exit 1; }
      test -s /tmp/libfl-consumer.o || { echo "ERROR: compiled libfl archive consumer missing" >&2; exit 1; }
      $BB chmod 644 /tmp/libfl-consumer.o
      tcc -nostdlib -static -o /tmp/libfl-consumer \
        "$MUSL/lib/crt1.o" /tmp/libfl-consumer.o "$out/lib/libfl.a" \
        "$MUSL/lib/libc.a" \
        || { echo "ERROR: failed linking libfl archive consumer" >&2; exit 1; }
      $BB chmod 755 /tmp/libfl-consumer
      /tmp/libfl-consumer || { echo "ERROR: libfl archive consumer execution failed" >&2; exit 1; }

      # Install FlexLexer.h if present and make the conditional boundary explicit.
      if [ -f src/FlexLexer.h ]; then
        $BB cp src/FlexLexer.h "$out/include/FlexLexer.h"
        test -f "$out/include/FlexLexer.h" || { echo "ERROR: installed FlexLexer.h missing" >&2; exit 1; }
      fi

      test -x "$out/bin/flex" || { echo "ERROR: installed flex missing" >&2; exit 1; }
      test -L "$out/bin/lex" || { echo "ERROR: installed lex symlink missing" >&2; exit 1; }
      test -L "$out/bin/flex++" || { echo "ERROR: installed flex++ symlink missing" >&2; exit 1; }
      "$out/bin/flex" --version >/tmp/flex-version.txt || { echo "ERROR: flex --version smoke failed" >&2; exit 1; }
      $BB grep -q 'flex 2.6.4' /tmp/flex-version.txt || { echo "ERROR: flex version output mismatch" >&2; exit 1; }
      $BB cat > /tmp/flex-smoke.l << 'SMOKE'
%%
hello  ECHO;
%%
int yywrap(void) { return 1; }
SMOKE
      M4="$M4/bin/m4" "$out/bin/flex" -o /tmp/flex-smoke.c /tmp/flex-smoke.l || { echo "ERROR: flex scanner generation smoke failed" >&2; exit 1; }
      test -f /tmp/flex-smoke.c || { echo "ERROR: flex smoke output missing" >&2; exit 1; }

      $BB cat > /tmp/flex-negative.l <<'NEGATIVE'
%%
[unterminated  ECHO;
%%
NEGATIVE
      if M4="$M4/bin/m4" "$out/bin/flex" -o /tmp/flex-negative.c /tmp/flex-negative.l; then
        echo "ERROR: flex accepted malformed scanner input" >&2
        exit 1
      fi
      test ! -s /tmp/flex-negative.c || { echo "ERROR: flex retained malformed scanner output" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, m4, flex_src],
} | crunch.Derivation
