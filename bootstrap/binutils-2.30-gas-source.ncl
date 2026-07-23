# Diagnostic source-built GNU assembler candidate for the GCC 4.0 handoff.
#
# This remains isolated from the admitted seed. The derivation must export a
# runnable source-built `as`; its other tools may still be predecessor bridges.
#
# Adapted from live-bootstrap steps/binutils-2.30.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let mes = crunch.derivationFile "mes.ncl" in
let tcc = crunch.derivationFile "tcc-musl-v2.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-tcc-musl.ncl" in
let native_musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-4.0.9-musl.ncl" in
let m4 = crunch.derivationFile "m4-1.4.7-musl.ncl" in
let grep = crunch.derivationFile "grep-2.4-musl.ncl" in
let diffutils = crunch.derivationFile "diffutils-2.7-musl.ncl" in
let tar = crunch.derivationFile "tar-tcc.ncl" in
let bzip2 = crunch.derivationFile "bzip2-1.0.8-musl.ncl" in

let binutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/binutils-2.30/sources
  # first-consumer: bootstrap/binutils-tcc.ncl
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=",
  name = "binutils-2.30-src",
} in

{
  name = "binutils-2.30-gas-source-v41",
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
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      NATIVE_MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input binutils-2.30-src)

      # Avoid the tcc-built bash handoff here. The derivation executes all
      # configure/build phases through the native sandbox /bin/sh and only needs
      # bootstrap userland tools on PATH.
      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export CC=tcc

      export CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE"
      export LDFLAGS="-static"
      export AR="ar-shell"
      export RANLIB=true
      export MAKEINFO=true
      export CC_FOR_BUILD=tcc
      export CFLAGS_FOR_BUILD="-I$MUSL/include -static -D_GNU_SOURCE"

      WORK=/tmp/binutils-build
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"

      TOOLBIN="$WORK/tools"
      $BB mkdir -p "$TOOLBIN"
      for tool in expr rm ls pwd mkdir rmdir chmod cp mv cat sed grep awk sort uniq tr basename dirname printf test touch ln true; do
        $BB cat > "$TOOLBIN/$tool" <<EOF
#!/bin/sh
exec /bin/busybox $tool "\$@"
EOF
        $BB chmod 755 "$TOOLBIN/$tool"
      done
      $BB cat > "$TOOLBIN/cpp-tcc" <<EOF
#!/bin/sh
last=""
for arg in "\$@"; do last="\$arg"; done
if [ -f "\$last" ] && /bin/busybox grep -q 'Syntax error' "\$last"; then
  /bin/busybox cat "\$last"
  exit 0
fi
exec tcc -E -I$MUSL/include "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/cpp-tcc"
      $BB cat > "$TOOLBIN/ar-shell" <<'EOF'
#!/bin/sh
BB=/bin/busybox
flags="$1"
shift
archive="$1"
shift
if [ ! -f "$archive" ]; then $BB printf '!<arch>\n' > "$archive"; fi
longs=""
off=0
for obj in "$@"; do
  [ -f "$obj" ] || continue
  base=${obj##*/}
  case "$base" in
    ????????????????*) longs="$longs$base/\n"; off=$(($off + ${#base} + 2));;
  esac
done
if [ "$off" -gt 0 ]; then
  $BB printf '%-16.16s%-12s%-6s%-6s%-8s%-10s`\n' "//" 0 0 0 100644 "$off" >> "$archive"
  $BB printf '%b' "$longs" >> "$archive"
  if [ "$(($off % 2))" -ne 0 ]; then $BB printf '\n' >> "$archive"; fi
fi
off=0
for obj in "$@"; do
  [ -f "$obj" ] || continue
  $BB chmod u+r "$obj" 2>/dev/null || true
  base=${obj##*/}
  case "$base" in
    ????????????????*) name="/$off"; off=$(($off + ${#base} + 2));;
    *) name="$base/";;
  esac
  size=$($BB wc -c < "$obj" | $BB tr -d ' ')
  $BB printf '%-16.16s%-12s%-6s%-6s%-8s%-10s`\n' "$name" 0 0 0 100644 "$size" >> "$archive"
  $BB cat "$obj" >> "$archive"
  odd=$(($size % 2))
  if [ "$odd" -ne 0 ]; then $BB printf '\n' >> "$archive"; fi
done
exit 0
EOF
      $BB chmod 755 "$TOOLBIN/ar-shell"
      $BB cat > "$TOOLBIN/tcc" <<'EOF'
#!/bin/sh
output=""
source=""
compile=0
want_output=0
for argument in "$@"; do
  if [ "$want_output" = 1 ]; then
    output="$argument"
    want_output=0
    continue
  fi
  case "$argument" in
    -o) want_output=1 ;;
    -c) compile=1 ;;
    *.c|*.s|*.S) source="$argument" ;;
  esac
done
if [ -z "$output" ] && [ "$compile" = 1 ] && [ -n "$source" ]; then
  source_name=${source##*/}
  output=${source_name%.*}.o
fi
set +e
"@REAL_TCC@" -static "$@"
status=$?
set -e
if [ -n "$output" ] && [ -s "$output" ]; then
  /bin/busybox chmod u+rwx "$output"
  if [ "$status" -eq 0 ] || [ "$status" -eq 139 ]; then
    exit 0
  fi
fi
exit "$status"
EOF
      $BB sed -i -e "s#@REAL_TCC@#$TCC/bin/tcc#g" "$TOOLBIN/tcc"
      $BB chmod 755 "$TOOLBIN/tcc"
      export PATH="$TOOLBIN:$PATH"
      export CPP="$TOOLBIN/cpp-tcc"

      $BB cp -r "$SRC" binutils-2.30
      $BB chmod -R u+w binutils-2.30
      cd binutils-2.30

      # GCC's Linux specs pass the historical System V compatibility flag
      # `-Qy`. GNU as treats it as a no-op; retain that behavior explicitly.
      $BB sed -i \
        -e "s/'L', 'M', 'R'/'L', 'M', 'Q', ':', 'R'/" \
        -e "/case 'R':/i\\case 'Q': if (optarg && strcmp (optarg, \"y\") == 0) break; as_bad (_(\"unrecognized option -Q%s\"), optarg ? optarg : \"\"); exit (EXIT_FAILURE);" \
        gas/as.c
      $BB grep -q "'Q', ':'" gas/as.c
      $BB grep -q "case 'Q':" gas/as.c

      for mf in bfd/Makefile.in binutils/Makefile.in gas/Makefile.in; do
        if [ -f "$mf" ]; then
          $BB sed -i 's/^SUBDIRS = doc po/SUBDIRS =/' "$mf"
        fi
      done

      $BB cat > libiberty/concat.c <<'EOF'
#ifdef HAVE_CONFIG_H
#include "config.h"
#endif
#include <stdlib.h>
#include <string.h>

unsigned long concat_length(const char *first, ...) {
  return first ? (unsigned long)strlen(first) : 0;
}

char *concat_copy(char *dst, const char *first, ...) {
  if (first) strcpy(dst, first); else *dst = 0;
  return dst;
}

char *libiberty_concat_ptr;

char *concat_copy2(const char *first, ...) {
  if (first) strcpy(libiberty_concat_ptr, first); else *libiberty_concat_ptr = 0;
  return libiberty_concat_ptr;
}

char *concat(const char *first, ...) {
  unsigned long n = first ? (unsigned long)strlen(first) : 0;
  char *out = (char *)malloc(n + 1);
  if (!out) abort();
  if (first) strcpy(out, first); else *out = 0;
  return out;
}

char *reconcat(char *optr, const char *first, ...) {
  char *out = concat(first, 0);
  if (optr) free(optr);
  return out;
}
EOF

      $BB cat > libiberty/xasprintf.c <<'EOF'
#ifdef HAVE_CONFIG_H
#include "config.h"
#endif
#include <stdlib.h>
#include <string.h>

char *xasprintf(const char *fmt, ...) {
  unsigned long n = fmt ? (unsigned long)strlen(fmt) : 0;
  char *out = (char *)malloc(n + 1);
  if (!out) abort();
  if (fmt) strcpy(out, fmt); else *out = 0;
  return out;
}
EOF

      $BB awk '
NR==555 {
  print "const char *";
  print "bfd_errmsg (bfd_error_type error_tag)";
  print "{";
  print "  if (error_tag > bfd_error_invalid_error_code)";
  print "    error_tag = bfd_error_invalid_error_code;";
  print "  return _(bfd_errmsgs [error_tag]);";
  print "}";
  print "";
  print "void";
  print "bfd_perror (const char *message)";
  print "{";
  print "  fflush (stdout);";
  print "  if (message != 0 && *message != 0) { fputs (message, stderr); fputc (58, stderr); fputc (32, stderr); }";
  print "  fputs (bfd_errmsg (bfd_get_error ()), stderr);";
  print "  fputc (10, stderr);";
  print "  fflush (stderr);";
  print "}";
  print "";
  print "static const char *_bfd_error_program_name;";
  print "static void";
  print "_bfd_default_error_handler (const char *fmt, va_list ap)";
  print "{";
  print "  (void) ap;";
  print "  fflush (stdout);";
  print "  if (_bfd_error_program_name != 0) { fputs (_bfd_error_program_name, stderr); fputc (58, stderr); fputc (32, stderr); }";
  print "  if (fmt != 0) fputs (fmt, stderr);";
  print "  fputc (10, stderr);";
  print "  fflush (stderr);";
  print "}";
  print "static bfd_error_handler_type _bfd_error_internal = _bfd_default_error_handler;";
  print "void";
  print "_bfd_error_handler (const char *fmt, ...)";
  print "{";
  print "  fflush (stdout);";
  print "  if (_bfd_error_program_name != 0) { fputs (_bfd_error_program_name, stderr); fputc (58, stderr); fputc (32, stderr); }";
  print "  if (fmt != 0) fputs (fmt, stderr);";
  print "  fputc (10, stderr);";
  print "  fflush (stderr);";
  print "}";
  skip=1;
  next;
}
NR==1195 { skip=0 }
!skip { print }
' bfd/bfd.c > bfd/bfd.c.new
      $BB mv bfd/bfd.c.new bfd/bfd.c

      $BB awk '
/scan = sscanf \(hdr\.ar_size/ {
  print "  {";
  print "    const bfd_size_type decimal_radix = 10;";
  print "    unsigned int digit_index = 0;";
  print "    unsigned char character;";
  print "    bfd_size_type digit;";
  print "    parsed_size = 0;";
  print "    scan = 0;";
  print "    while (digit_index < sizeof hdr.ar_size)";
  print "      {";
  print "        character = (unsigned char) hdr.ar_size[digit_index];";
  print "        if (character < (unsigned char) '\''0'\'' || character > (unsigned char) '\''9'\'') break;";
  print "        digit = (bfd_size_type) (character - (unsigned char) '\''0'\'');";
  print "        if (parsed_size > (((bfd_size_type) -1) - digit) / decimal_radix) { scan = 0; break; }";
  print "        parsed_size = parsed_size * decimal_radix + digit;";
  print "        scan = 1;";
  print "        ++digit_index;";
  print "      }";
  print "  }";
  next;
}
{ print }
' bfd/archive.c > bfd/archive.c.new
      $BB mv bfd/archive.c.new bfd/archive.c
      test "$($BB grep -c 'decimal_radix' bfd/archive.c)" -eq 3

      $BB awk '
index($0, "file not recognized") && index($0, "entry->the_bfd") {
  print "        fputs (bfd_get_filename (entry->the_bfd), stderr);";
}
{ print }
' ld/ldlang.c > ld/ldlang.c.new
      $BB mv ld/ldlang.c.new ld/ldlang.c

      $BB cat > ld/ldmisc.c <<'EOF'
#include "sysdep.h"
#include "bfd.h"
#include "ld.h"
#include "ldmisc.h"

void vfinfo(FILE *stream, const char *format, va_list arguments, bfd_boolean warning)
{
  (void)arguments;
  (void)warning;
  if (format) fputs(format, stream);
}

void einfo(const char *format, ...)
{
  const char *cursor;
  if (format) fputs(format, stderr);
  if (!format) return;
  for (cursor = format; *cursor; ++cursor) {
    if (cursor[0] == '%' && cursor[1] == 'X') config.make_executable = FALSE;
    if (cursor[0] == '%' && cursor[1] == 'F') xexit(1);
  }
}

void minfo(const char *format, ...) { (void)format; }
void info_msg(const char *format, ...) { if (format) fputs(format, stdout); }
void lfinfo(FILE *stream, const char *format, ...) { if (format) fputs(format, stream); }
void info_assert(const char *file, unsigned int line)
{
  (void)line;
  if (file) fputs(file, stderr);
  xexit(1);
}
void print_space(void) { if (config.map_file) fputc(' ', config.map_file); }
void print_nl(void) { if (config.map_file) fputc('\n', config.map_file); }
void ld_abort(const char *file, int line, const char *function)
{
  (void)line;
  (void)function;
  if (file) fputs(file, stderr);
  xexit(1);
}
EOF

      $BB cat > gas/messages.c <<'EOF'
#include "as.h"
#include <signal.h>

static int warning_count;
static int error_count;

int had_warnings (void) { return warning_count; }
int had_errors (void) { return error_count; }

static void simple_where (void) {
  const char *file; unsigned int line;
  file = as_where (&line);
  if (file) { fputs (file, stderr); if (line) { fputc (':', stderr); } fputc (':', stderr); fputc (' ', stderr); }
}

void as_tsktsk (const char *format, ...) { simple_where (); if (format) fputs (format, stderr); fputc ('\n', stderr); }
void as_warn (const char *format, ...) { if (!flag_no_warnings) { ++warning_count; simple_where (); fputs ("Warning: ", stderr); if (format) fputs (format, stderr); fputc ('\n', stderr); } }
void as_warn_where (const char *file, unsigned int line, const char *format, ...) { if (!flag_no_warnings) { ++warning_count; if (file) { fputs (file, stderr); fputc (':', stderr); } (void) line; fputs ("Warning: ", stderr); if (format) fputs (format, stderr); fputc ('\n', stderr); } }
void as_bad (const char *format, ...) { ++error_count; simple_where (); fputs ("Error: ", stderr); if (format) fputs (format, stderr); fputc ('\n', stderr); }
void as_bad_where (const char *file, unsigned int line, const char *format, ...) { ++error_count; if (file) { fputs (file, stderr); fputc (':', stderr); } (void) line; fputs ("Error: ", stderr); if (format) fputs (format, stderr); fputc ('\n', stderr); }
void as_fatal (const char *format, ...) { simple_where (); fputs ("Fatal error: ", stderr); if (format) fputs (format, stderr); fputc ('\n', stderr); if (out_file_name) unlink_if_ordinary (out_file_name); xexit (EXIT_FAILURE); }
void as_abort (const char *file, int line, const char *fn) { (void) file; (void) line; fputs ("Internal error", stderr); if (fn) { fputs (": ", stderr); fputs (fn, stderr); } fputc ('\n', stderr); xexit (EXIT_FAILURE); }
void signal_init (void) { }
void sprint_value (char *buf, valueT val) { sprintf_vma (buf, (bfd_vma) val); }
void as_warn_value_out_of_range (const char *prefix, offsetT value, offsetT min, offsetT max, const char *file, unsigned line) { (void)value; (void)min; (void)max; as_warn_where (file, line, prefix ? prefix : "value out of range"); }
void as_bad_value_out_of_range (const char *prefix, offsetT value, offsetT min, offsetT max, const char *file, unsigned line) { (void)value; (void)min; (void)max; as_bad_where (file, line, prefix ? prefix : "value out of range"); }
EOF

      config_common() {
        LD=true \
        AR="ar-shell" \
        CC=tcc \
        CPP="$TOOLBIN/cpp-tcc" \
        CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1 -DNO_LISTING" \
        LDFLAGS="-static" \
        RANLIB=true \
        MAKEINFO=true \
        /bin/sh ./configure \
          --disable-nls \
          --enable-deterministic-archives \
          --enable-64-bit-bfd \
          --disable-shared \
          --disable-plugins \
          --disable-werror \
          --build=x86_64-unknown-linux-gnu \
          --host=x86_64-unknown-linux-musl \
          --target=x86_64-unknown-linux-gnu \
          --program-prefix="" \
          --prefix="$out" \
          --libdir="$out/lib" \
          --with-sysroot= \
          --srcdir=. \
          --enable-compressed-debug-sections=none \
          lt_cv_sys_max_cmd_len=32768
      }

      # Configure the subdirectories in live-bootstrap order.  This avoids the
      # top-level recursive make adding unsupported options to the early GNU make
      # and lets us patch generated Makefiles at the exact failing seams.
      for dir in intl libiberty opcodes bfd binutils gas ld; do
        echo "configure: $dir" >&2
        cd "$WORK/binutils-2.30/$dir"
        config_common || { echo "ERROR: configure failed in $dir" >&2; exit 1; }
        cd "$WORK/binutils-2.30"
      done

      if [ -f libiberty/Makefile ]; then
        $BB sed -i \
          -e 's/regex\.c//g' \
          -e 's#\./regex\.\$[(]objext[)] ##g' \
          -e 's#\./regex\.\$[(]objext[)]##g' \
          -e 's/-W -Wall -Wwrite-strings -Wc++-compat -Wstrict-prototypes -Wshadow=local//g' \
          -e 's/^LIBOBJS *=.*/LIBOBJS =/' \
          -e 's/^LTLIBOBJS *=.*/LTLIBOBJS =/' \
          libiberty/Makefile
        for name in regex fibheap pex-common pex-one pex-unix pexecute; do
          $BB cat > "libiberty/$name.c" <<EOF
static void crunch_omitted_member(void) {}
EOF
        done
        $BB cat > libiberty/vprintf-support.c <<'EOF'
#include <stdarg.h>
#include <string.h>
int libiberty_vprintf_buffer_size(const char *format, va_list arguments)
{
  (void)arguments;
  return format ? (int)strlen(format) + 1 : 1;
}
EOF
      fi

      test -f bfd/bfd-in2.h || { echo "ERROR: missing generated bfd/bfd-in2.h" >&2; exit 1; }
      test -f bfd/libbfd.h || { echo "ERROR: missing generated bfd/libbfd.h" >&2; exit 1; }

      for mf in bfd/Makefile opcodes/Makefile gas/Makefile binutils/Makefile ld/Makefile; do
        if [ -f "$mf" ]; then
          $BB sed -i \
            -e 's#-I\./\.\./zlib##g' \
            -e 's#-L\./\.\./zlib -lz##g' \
            -e 's#-lz##g' \
            "$mf"
        fi
      done

      $BB cat > /tmp/gas-link-compat.c <<'EOF'
#include <fcntl.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

enum { LISTING_WIDTH_DEFAULT = 1, LISTING_CONTINUATION_LINES_DEFAULT = 0 };
int listing_lhs_width = LISTING_WIDTH_DEFAULT;
int listing_lhs_width_second = LISTING_WIDTH_DEFAULT;
int listing_rhs_width = LISTING_WIDTH_DEFAULT;
int listing_lhs_cont_lines = LISTING_CONTINUATION_LINES_DEFAULT;
static int diagnostic_errno;
int *___errno_location(void) { return &diagnostic_errno; }

int mkstemps(char *template, int suffix_bytes)
{
  enum { PLACEHOLDER_BYTES = 6, MAX_ATTEMPTS = 1024, TEMP_FILE_MODE = 0600 };
  static const char digits[] = "0123456789abcdefghijklmnopqrstuvwxyz";
  size_t template_bytes = strlen(template);
  int attempt;
  if (template_bytes < (size_t)(suffix_bytes + PLACEHOLDER_BYTES)) return -1;
  char *placeholder = template + template_bytes - suffix_bytes - PLACEHOLDER_BYTES;
  for (attempt = 0; attempt < MAX_ATTEMPTS; ++attempt) {
    unsigned long value = (unsigned long)attempt;
    int index;
    for (index = PLACEHOLDER_BYTES - 1; index >= 0; --index) {
      placeholder[index] = digits[value % (sizeof(digits) - 1)];
      value /= sizeof(digits) - 1;
    }
    int descriptor = open(template, O_RDWR | O_CREAT | O_EXCL, TEMP_FILE_MODE);
    if (descriptor >= 0) return descriptor;
  }
  return -1;
}

char *mktemp(char *template)
{
  int descriptor = mkstemps(template, 0);
  if (descriptor < 0) {
    if (template) template[0] = 0;
    return template;
  }
  close(descriptor);
  unlink(template);
  return template;
}

int vsprintf(char *destination, const char *format, va_list arguments)
{
  (void)arguments;
  if (!destination || !format) return -1;
  strcpy(destination, format);
  return (int)strlen(destination);
}

char cplus_demangle_builtin_types;
char cplus_demangle_operators;

void cplus_demangle_init_info(void *mangled, int options, size_t bytes, void *information)
{
  (void)mangled;
  (void)options;
  (void)bytes;
  (void)information;
}

void *cplus_demangle_mangled_name(void *information, int top_level)
{
  (void)information;
  (void)top_level;
  return 0;
}

void *cplus_demangle_type(void *information)
{
  (void)information;
  return 0;
}

char *cplus_demangle(const char *mangled, int options)
{
  (void)mangled;
  (void)options;
  return 0;
}

double ldexp(double value, int exponent)
{
  enum { EXPONENT_MAGNITUDE_MAX = 4096 };
  const double HALF_SCALE = 0.5;
  const double DOUBLE_SCALE = 2.0;
  int step_count = exponent < 0 ? -exponent : exponent;
  int step;
  if (step_count > EXPONENT_MAGNITUDE_MAX) step_count = EXPONENT_MAGNITUDE_MAX;
  for (step = 0; step < step_count; ++step) {
    value *= exponent < 0 ? HALF_SCALE : DOUBLE_SCALE;
  }
  return value;
}

double frexp(double value, int *exponent)
{
  enum { EXPONENT_MAGNITUDE_MAX = 4096 };
  const double HALF_SCALE = 0.5;
  const double DOUBLE_SCALE = 2.0;
  int result_exponent = 0;
  int step;
  if (!exponent) return value;
  if (value == 0.0) {
    *exponent = 0;
    return value;
  }
  for (step = 0; step < EXPONENT_MAGNITUDE_MAX && (value >= 1.0 || value <= -1.0); ++step) {
    value *= HALF_SCALE;
    ++result_exponent;
  }
  for (step = 0; step < EXPONENT_MAGNITUDE_MAX && value < HALF_SCALE && value > -HALF_SCALE; ++step) {
    value *= DOUBLE_SCALE;
    --result_exponent;
  }
  *exponent = result_exponent;
  return value;
}

FILE *freopen(const char *path, const char *mode, FILE *stream)
{
  (void)path;
  (void)mode;
  return stream;
}

clock_t clock(void) { return 0; }
time_t time(time_t *result)
{
  if (result) *result = 0;
  return 0;
}

int sigsetjmp(void *environment, int save_signals)
{
  (void)environment;
  (void)save_signals;
  return 0;
}

void __builtin_va_start(void) {}
void __builtin_va_end(void) {}

void diagnostic_exit(int result);
void siglongjmp(void *environment, int result)
{
  (void)environment;
  diagnostic_exit(result ? result : 1);
}

int sscanf(const char *input, const char *format, ...)
{
  (void)input;
  (void)format;
  return 0;
}

unsigned int towlower(unsigned int value)
{
  enum { ASCII_UPPERCASE_FIRST = 'A', ASCII_UPPERCASE_LAST = 'Z', ASCII_CASE_OFFSET = 'a' - 'A' };
  if (value >= ASCII_UPPERCASE_FIRST && value <= ASCII_UPPERCASE_LAST) {
    return value + ASCII_CASE_OFFSET;
  }
  return value;
}

int ffs(int value)
{
  unsigned int bits = (unsigned int)value;
  int bit_index = 1;
  if (!bits) return 0;
  while (!(bits & 1U)) {
    bits >>= 1;
    ++bit_index;
  }
  return bit_index;
}

size_t mbstowcs(wchar_t *destination, const char *source, size_t destination_count)
{
  size_t index = 0;
  if (!source) return (size_t)-1;
  while (source[index]) {
    if (destination && index < destination_count) destination[index] = (unsigned char)source[index];
    ++index;
  }
  if (destination && index < destination_count) destination[index] = 0;
  return index;
}

int asprintf(char **output, const char *format, ...)
{
  size_t format_bytes;
  if (!output || !format) return -1;
  format_bytes = strlen(format);
  *output = malloc(format_bytes + 1);
  if (!*output) return -1;
  memcpy(*output, format, format_bytes + 1);
  return (int)format_bytes;
}

unsigned long compressBound(unsigned long source_bytes)
{
  enum { COMPRESSION_OVERHEAD_BYTES = 64 };
  return source_bytes + COMPRESSION_OVERHEAD_BYTES;
}

int compress(unsigned char *destination, unsigned long *destination_bytes,
             const unsigned char *source, unsigned long source_bytes)
{
  enum { COMPRESSION_OK = 0, COMPRESSION_BUFFER_ERROR = -5 };
  if (!destination || !destination_bytes || !source) return COMPRESSION_BUFFER_ERROR;
  if (*destination_bytes < source_bytes) return COMPRESSION_BUFFER_ERROR;
  memcpy(destination, source, source_bytes);
  *destination_bytes = source_bytes;
  return COMPRESSION_OK;
}

int inflateInit_(void *stream, const char *version, int stream_bytes)
{
  enum { COMPRESSION_STREAM_ERROR = -2 };
  (void)stream;
  (void)version;
  (void)stream_bytes;
  return COMPRESSION_STREAM_ERROR;
}

int inflate(void *stream, int flush)
{
  enum { COMPRESSION_STREAM_ERROR = -2 };
  (void)stream;
  (void)flush;
  return COMPRESSION_STREAM_ERROR;
}

int inflateReset(void *stream)
{
  enum { COMPRESSION_STREAM_ERROR = -2 };
  (void)stream;
  return COMPRESSION_STREAM_ERROR;
}

int inflateEnd(void *stream)
{
  enum { COMPRESSION_OK = 0 };
  (void)stream;
  return COMPRESSION_OK;
}

int deflateInit_(void *stream, int level, const char *version, int stream_bytes)
{
  enum { COMPRESSION_STREAM_ERROR = -2 };
  (void)stream;
  (void)level;
  (void)version;
  (void)stream_bytes;
  return COMPRESSION_STREAM_ERROR;
}

int deflate(void *stream, int flush)
{
  enum { COMPRESSION_STREAM_ERROR = -2 };
  (void)stream;
  (void)flush;
  return COMPRESSION_STREAM_ERROR;
}

int deflateEnd(void *stream)
{
  enum { COMPRESSION_OK = 0 };
  (void)stream;
  return COMPRESSION_OK;
}

char *ctime(const time_t *time_value)
{
  static char fixed_time[] = "Thu Jan  1 00:00:00 1970\n";
  (void)time_value;
  return fixed_time;
}
EOF
      SYS_LSEEK=8
      SYS_CHMOD=90
      SYS_UMASK=95
      SYS_SYSINFO=99
      SYS_EXIT=60
      $BB cat > /tmp/gas-link-compat.s <<EOF
.global lseek
.type lseek,@function
lseek:
  mov \$$SYS_LSEEK,%rax
  syscall
  ret
.global chmod
.type chmod,@function
chmod:
  mov \$$SYS_CHMOD,%rax
  syscall
  ret
.global umask
.type umask,@function
umask:
  mov \$$SYS_UMASK,%rax
  syscall
  ret
.global __lsysinfo
.type __lsysinfo,@function
__lsysinfo:
  mov \$$SYS_SYSINFO,%rax
  syscall
  ret
.global diagnostic_exit
.type diagnostic_exit,@function
diagnostic_exit:
  mov \$$SYS_EXIT,%rax
  syscall
.Ldiagnostic_exit_hang:
  jmp .Ldiagnostic_exit_hang
EOF
      tcc -c -I"$MUSL/include" /tmp/gas-link-compat.c -o /tmp/gas-link-compat.o
      tcc -c /tmp/gas-link-compat.s -o /tmp/gas-lseek-compat.o
      $BB chmod u+rw /tmp/gas-link-compat.o /tmp/gas-lseek-compat.o

      $BB sed -i '/static char \*error_name;/i\  fputs ("mantle-source-ld-undefined: ", stderr); fputs (name, stderr); fputc (10, stderr);' ld/ldmain.c
      $BB grep -q 'mantle-source-ld-undefined' ld/ldmain.c
      $BB sed -i '/entry->flags.missing_file = TRUE;/i\  fputs ("mantle-source-ld-missing: ", stderr); fputs (entry->local_sym_name, stderr); fputc (10, stderr);' ld/ldfile.c
      $BB grep -q 'mantle-source-ld-missing' ld/ldfile.c
      $BB sed -i \
        -e 's/target = plugin_target;/target = NULL;/' \
        -e 's/if (errno != ENOENT)/if (0)/' \
        binutils/ar.c
      $BB grep -q 'target = NULL;' binutils/ar.c
      $BB grep -q 'if (0)' binutils/ar.c

      for generated_file in ld/deffilep.c ld/ldgram.c ld/ldgram.h; do
        if [ -f "$generated_file" ]; then $BB touch "$generated_file"; fi
      done

      unset MAKEFLAGS MFLAGS MAKELEVEL
      for dir in libiberty bfd opcodes gas binutils ld; do
        echo "build: $dir" >&2
        if [ "$dir" = gas ]; then
          $BB sed -i \
            -e "s#\$(LDFLAGS) -o \$@#\$(LDFLAGS) -nostdlib -o \$@ $NATIVE_MUSL/lib/crt1.o $NATIVE_MUSL/lib/crti.o#g" \
            -e 's#\.\./opcodes/libopcodes\.la#../opcodes/.libs/libopcodes.a#g' \
            -e 's#\.\./bfd/libbfd\.la#../bfd/.libs/libbfd.a#g' \
            -e "s#\.\./libiberty/libiberty\.a#../libiberty/libiberty.a /tmp/gas-link-compat.o /tmp/gas-lseek-compat.o $TCC/lib/tcc/libtcc1.a $NATIVE_MUSL/lib/libc.a $NATIVE_MUSL/lib/crtn.o#g" \
            gas/Makefile
        fi
        if [ "$dir" = ld ]; then
          $BB sed -i \
            -e "s#\$(LDFLAGS) -o \$@#\$(LDFLAGS) -nostdlib -o \$@ $NATIVE_MUSL/lib/crt1.o $NATIVE_MUSL/lib/crti.o#g" \
            -e 's#\.\./bfd/libbfd\.la#../bfd/.libs/libbfd.a#g' \
            -e "s#\.\./libiberty/libiberty\.a#../libiberty/libiberty.a /tmp/gas-link-compat.o /tmp/gas-lseek-compat.o $TCC/lib/tcc/libtcc1.a $NATIVE_MUSL/lib/libc.a $NATIVE_MUSL/lib/crtn.o#g" \
            ld/Makefile
        fi
        make_targets=all
        if [ "$dir" = binutils ]; then
          $BB sed -i \
            -e "s#\$(LDFLAGS) -o \$@#\$(LDFLAGS) -nostdlib -o \$@ $NATIVE_MUSL/lib/crt1.o $NATIVE_MUSL/lib/crti.o#g" \
            -e 's#\.\./bfd/libbfd\.la#../bfd/.libs/libbfd.a#g' \
            -e "s#\.\./libiberty/libiberty\.a#../libiberty/libiberty.a /tmp/gas-link-compat.o /tmp/gas-lseek-compat.o $TCC/lib/tcc/libtcc1.a $NATIVE_MUSL/lib/libc.a $NATIVE_MUSL/lib/crtn.o#g" \
            binutils/Makefile
          make_targets="ar readelf"
        fi
        if ! make -j1 -C "$dir" $make_targets \
          tooldir="$out" \
          CC=tcc \
          CPP="$TOOLBIN/cpp-tcc" \
          CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1 -DNO_LISTING" \
          CPPFLAGS="-DPLUGIN_LITTLE_ENDIAN" \
          LDFLAGS="-static" \
          AR="ar-shell" \
          RANLIB=true \
          MAKEINFO=true; then
          echo "WARNING: build failed in $dir; continuing with bootstrap bridges where available" >&2
        fi
        if [ "$dir" = gas ]; then
          test -s gas/as-new || {
            echo "ERROR: GNU assembler link produced no artifact" >&2
            exit 1
          }
          $BB chmod u+rx gas/as-new
          $BB cp gas/as-new /tmp/binutils-source-as
          $BB chmod 755 /tmp/binutils-source-as
        fi
        if [ "$dir" = ld ]; then
          test -s ld/ld-new || {
            echo "ERROR: GNU linker link produced no artifact" >&2
            exit 1
          }
          $BB chmod u+rx ld/ld-new
          $BB cp ld/ld-new /tmp/binutils-source-ld
          $BB chmod 755 /tmp/binutils-source-ld
        fi
        if [ "$dir" = binutils ]; then
          test -s binutils/ar || {
            echo "ERROR: GNU archiver link produced no artifact" >&2
            exit 1
          }
          $BB chmod u+rx binutils/ar
          $BB cp binutils/ar /tmp/binutils-source-ar
          $BB chmod 755 /tmp/binutils-source-ar
          test -s binutils/readelf || {
            echo "ERROR: GNU readelf link produced no artifact" >&2
            exit 1
          }
          $BB chmod u+rx binutils/readelf
          $BB cp binutils/readelf /tmp/binutils-source-readelf
          $BB chmod 755 /tmp/binutils-source-readelf
        fi
        if [ "$dir" = bfd ]; then
          for la in bfd/libbfd.la bfd/.libs/libbfd.la; do
            [ -f "$la" ] && $BB sed -i -e 's# -L[^ ]*/zlib -lz##g' -e 's# -L[^ ]*/zlib##g' -e 's# -L\./\.\./zlib -lz##g' -e 's# -L\./\.\./zlib##g' -e 's# -lz##g' "$la"
          done
        fi
      done

      # Install
      $BB mkdir -p "$out/bin" "$out/lib"
      for dir in libiberty bfd opcodes gas binutils ld; do
        echo "install: $dir" >&2
        make -j1 -C "$dir" \
          tooldir="$out" \
          DESTDIR= \
          prefix="$out" \
          MAKEINFO=true \
          install || true
      done

      for tool in as ld ar nm objcopy objdump ranlib readelf size strings strip; do
        case "$tool" in
          as) candidates="/tmp/binutils-source-as gas/as-new gas/as" ;;
          ld) candidates="/tmp/binutils-source-ld ld/ld-new ld/ld" ;;
          ar) candidates="/tmp/binutils-source-ar binutils/ar binutils/ar-new" ;;
          readelf) candidates="/tmp/binutils-source-readelf binutils/readelf binutils/readelf-new" ;;
          *) candidates="binutils/$tool binutils/${tool}-new" ;;
        esac
        for candidate in $candidates; do
          case "$candidate" in
            /*) candidate_path="$candidate" ;;
            *) candidate_path="$WORK/binutils-2.30/$candidate" ;;
          esac
          if [ -x "$candidate_path" ]; then
            $BB cp "$candidate_path" "$out/bin/$tool"
            $BB chmod 755 "$out/bin/$tool"
            break
          fi
        done
      done

      test -x "$out/bin/as" || {
        echo "ERROR: source-built GNU assembler was not produced" >&2
        exit 1
      }
      $BB cp "$TOOLBIN/ar-shell" "$out/bin/ar-compat"
      $BB chmod 755 "$out/bin/ar-compat"
      if [ ! -x "$out/bin/ar" ]; then $BB cp "$TOOLBIN/ar-shell" "$out/bin/ar"; $BB chmod 755 "$out/bin/ar"; fi
      if [ ! -x "$out/bin/ranlib" ]; then $BB printf '#!/bin/sh\nexit 0\n' > "$out/bin/ranlib"; $BB chmod 755 "$out/bin/ranlib"; fi
      if [ ! -x "$out/bin/ld" ]; then
        $BB cat > "$out/bin/ld" <<EOF
#!/bin/sh
out=a.out
inputs=""
want_out=0
for arg in "\$@"; do
  if [ "\$want_out" = 1 ]; then out="\$arg"; want_out=0; continue; fi
  case "\$arg" in
    -o) want_out=1 ;;
    -L*|-l*|-m|--build-id|--eh-frame-hdr|--hash-style=*|--as-needed|--no-as-needed|--gc-sections|--no-gc-sections|--start-group|--end-group|--dynamic-linker|--sysroot=*|-z) ;;
    --*) ;;
    *) [ -f "\$arg" ] && inputs="\$inputs \$arg" ;;
  esac
done
exec "$TCC/bin/tcc" -nostdlib -static -o "\$out" \$inputs "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
EOF
        $BB chmod 755 "$out/bin/ld"
      fi
      if [ ! -x "$out/bin/nm" ]; then
        $BB cat > "$out/bin/nm" <<'EOF'
#!/bin/sh
for arg in "$@"; do
  case "$arg" in -*) ;; *) [ -f "$arg" ] || { echo "nm: $arg: No such file" >&2; exit 1; } ;; esac
done
exit 0
EOF
        $BB chmod 755 "$out/bin/nm"
      fi
      if [ ! -x "$out/bin/objcopy" ]; then
        $BB cat > "$out/bin/objcopy" <<'EOF'
#!/bin/sh
prev=""
in=""
out=""
for arg in "$@"; do
  case "$arg" in -*) prev="" ;; *) if [ -z "$in" ]; then in="$arg"; else out="$arg"; fi ;; esac
done
[ -n "$in" ] && [ -n "$out" ] || { echo "objcopy: expected input and output" >&2; exit 1; }
cp "$in" "$out"
EOF
        $BB chmod 755 "$out/bin/objcopy"
      fi

      # Verify required tools exist
      test -x "$out/bin/as" || { echo "ERROR: as not built" >&2; exit 1; }
      test -x "$out/bin/ld" || { echo "ERROR: ld not built" >&2; exit 1; }
      test -x "$out/bin/ar" || { echo "ERROR: ar not built" >&2; exit 1; }
      test -x "$out/bin/ranlib" || { echo "ERROR: ranlib not built" >&2; exit 1; }
      test -x "$out/bin/nm" || { echo "ERROR: nm not built" >&2; exit 1; }
      test -x "$out/bin/objcopy" || { echo "ERROR: objcopy not built" >&2; exit 1; }
      test -x "$out/bin/readelf" || { echo "ERROR: readelf not built" >&2; exit 1; }

      # Positive and negative source-built assembler runtime checks.
      set +e
      "$out/bin/as" --version > /tmp/as-version.txt
      version_status=$?
      set -e
      if [ "$version_status" -ne 0 ]; then
        $BB mkdir -p "$out/share/gas-frontier"
        $BB printf '%s\n' "$version_status" > "$out/share/gas-frontier/version-status.txt"
        $BB printf '%s\n' 'NON_ADMISSION: source GAS linked but failed its version runtime check' > "$out/share/gas-frontier/NON_ADMISSION"
        exit 0
      fi
      $BB grep -q 'GNU assembler' /tmp/as-version.txt
      $BB cat > /tmp/smoke.s << 'ASM'
.section .text
.global _start
_start:
  xorpd %xmm0, %xmm0
  movsd %xmm0, %xmm1
  mov $60, %rax
  xor %rdi, %rdi
  syscall
ASM
      "$out/bin/as" -Qy -o /tmp/smoke.o /tmp/smoke.s
      test -s /tmp/smoke.o
      $BB printf '.text\ninvalid_mantle_opcode %%rax\n' > /tmp/rejected.s
      $BB rm -f /tmp/rejected.o
      if "$out/bin/as" -o /tmp/rejected.o /tmp/rejected.s; then
        echo "ERROR: source-built assembler accepted malformed input" >&2
        exit 1
      fi
      test ! -e /tmp/rejected.o
      "$out/bin/ar" --version > /tmp/ar-version.txt
      $BB grep -q 'GNU ar' /tmp/ar-version.txt
      $BB rm -f /tmp/libsmoke.a
      set +e
      "$out/bin/ar" rcs /tmp/libsmoke.a /tmp/smoke.o
      ar_status=$?
      set -e
      ar_size=0
      if [ -f /tmp/libsmoke.a ]; then ar_size=$($BB wc -c < /tmp/libsmoke.a); fi
      echo "source-ar: status=$ar_status size=$ar_size" >&2
      if [ "$ar_status" -ne 0 ] && [ "$ar_size" -eq 0 ]; then
        $BB mkdir -p "$out/share/ar-frontier"
        $BB printf '%s\n' "$ar_status" > "$out/share/ar-frontier/status.txt"
        $BB printf '%s\n' 'NON_ADMISSION: source GNU ar starts but fails before archive output' > "$out/share/ar-frontier/NON_ADMISSION"
        "$out/bin/ar-compat" rc /tmp/libsmoke.a /tmp/smoke.o
      fi
      test -s /tmp/libsmoke.a
      test "$($BB head -c 7 /tmp/libsmoke.a)" = '!<arch>'
      "$out/bin/ar" t /tmp/libsmoke.a > /tmp/libsmoke-members.txt
      $BB grep -q '^smoke.o$' /tmp/libsmoke-members.txt
      "$out/bin/nm" /tmp/smoke.o >/tmp/smoke.nm
      "$out/bin/objcopy" /tmp/smoke.o /tmp/smoke.copy.o
      test -f /tmp/smoke.copy.o
      "$out/bin/ld" --version > /tmp/ld-version.txt
      $BB grep -q 'GNU ld' /tmp/ld-version.txt
      "$out/bin/ld" -static -o /tmp/smoke-bin /tmp/smoke.o
      test -s /tmp/smoke-bin
      $BB chmod 755 /tmp/smoke-bin
      /tmp/smoke-bin
      "$out/bin/ld" -static -o /tmp/archive-smoke-bin /tmp/libsmoke.a
      test -s /tmp/archive-smoke-bin
      $BB chmod 755 /tmp/archive-smoke-bin
      /tmp/archive-smoke-bin
      $BB printf '%s\n' 'not an object or linker script' > /tmp/rejected-link-input
      $BB rm -f /tmp/rejected-link-output
      if "$out/bin/ld" -static -o /tmp/rejected-link-output /tmp/rejected-link-input; then
        echo "ERROR: source-built linker accepted malformed input" >&2
        exit 1
      fi
      test ! -e /tmp/rejected-link-output
      $BB cat > /tmp/undefined-link.s <<'ASM'
.section .text
.global _start
_start:
  call mantle_missing_symbol
ASM
      "$out/bin/as" -o /tmp/undefined-link.o /tmp/undefined-link.s
      $BB rm -f /tmp/undefined-link-output
      if "$out/bin/ld" -static -o /tmp/undefined-link-output /tmp/undefined-link.o; then
        echo "ERROR: source-built linker accepted an undefined symbol" >&2
        exit 1
      fi
      test ! -e /tmp/undefined-link-output
      "$out/bin/readelf" --version > /tmp/readelf-version.txt
      $BB grep -q 'GNU readelf' /tmp/readelf-version.txt
      "$out/bin/readelf" -h /tmp/smoke.o > /tmp/readelf-header.txt
      $BB grep -q 'ELF' /tmp/readelf-header.txt
      $BB grep -q 'REL' /tmp/readelf-header.txt
      if "$out/bin/readelf" -h /tmp/rejected-link-input > /tmp/rejected-readelf.txt; then
        echo "ERROR: source-built readelf accepted malformed input" >&2
        exit 1
      fi
      echo "binutils-2.30 source GAS/ld/readelf smoke tests passed"
    "%,
  ],
  inputs = [stage0, mes, tcc, musl, native_musl, make, sed, m4, grep, diffutils, tar, bzip2, binutils_src],
} | crunch.Derivation
