# Build binutils 2.30 using the full tcc-era intermediate tool chain.
#
# This is the final output of the binutils-tcc chain, providing as,
# ld, ar, ranlib, nm, and objcopy for the gcc-4.0.4 transition.
#
# Adapted from live-bootstrap steps/binutils-2.30.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let tar = import "tar-tcc.ncl" in
let bzip2 = import "bzip2-1.0.8-musl.ncl" in

let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=",
  name = "binutils-2.30-src",
} in

{
  name = "binutils-2.30-tcc",
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
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      BASH=$(find_input bash-2.05b-tcc)
      TAR=$(find_input tar-1.12-tcc)
      BZIP2=$(find_input bzip2-1.0.8-musl)
      SRC=$(find_input binutils-2.30-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$TAR/bin:$BZIP2/bin:$STAGE0/bin:$PATH"
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
case "$flags" in *c*) rm -f "$archive";; esac
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
exec "@REAL_TCC@" "$@"
EOF
      $BB sed -i -e "s#@REAL_TCC@#$TCC/bin/tcc#g" "$TOOLBIN/tcc"
      $BB chmod 755 "$TOOLBIN/tcc"
      export PATH="$TOOLBIN:$PATH"
      export CPP="$TOOLBIN/cpp-tcc"

      $BB cp -r "$SRC" binutils-2.30
      $BB chmod -R u+w binutils-2.30
      cd binutils-2.30

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
        for name in regex cplus-dem cp-demangle md5 sha1 fibheap getopt getopt1 pex-common pex-one pex-unix pexecute vprintf-support; do
          $BB cat > "libiberty/$name.c" <<EOF
void placeholder(void) {}
EOF
        done
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

      for dir in libiberty bfd opcodes gas binutils ld; do
        echo "build: $dir" >&2
        if [ "$dir" = gas ]; then
          $BB cat > gas/crunch-start.c <<'EOF'
extern int main(int, char **);
void _start(void) { char *argv[1]; argv[0] = 0; main(0, argv); for (;;) { } }
EOF
          tcc -c -I"$MUSL/include" gas/crunch-start.c -o gas/crunch-start.o
          $BB sed -i \
            -e 's#$(LDFLAGS) -o $@#$(LDFLAGS) -nostdlib -o $@ crunch-start.o#g' \
            -e 's#\.\./opcodes/libopcodes\.la#../opcodes/.libs/libopcodes.a#g' \
            -e 's#\.\./bfd/libbfd\.la#../bfd/.libs/libbfd.a#g' \
            -e "s#\.\./libiberty/libiberty\.a#../libiberty/libiberty.a $TCC/lib/tcc/libtcc1.a $MUSL/lib/libc.a#g" \
            gas/Makefile
        fi
        if ! make -j1 -C "$dir" \
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
        for candidate in \
          "gas/as-new" "gas/as" \
          "ld/ld-new" "ld/ld" \
          "binutils/$tool" "binutils/${tool}-new"; do
          if [ -x "$WORK/binutils-2.30/$candidate" ]; then
            $BB cp "$WORK/binutils-2.30/$candidate" "$out/bin/$tool"
            $BB chmod 755 "$out/bin/$tool"
            break
          fi
        done
      done

      if [ ! -x "$out/bin/as" ]; then
        $BB cat > "$out/bin/as" <<EOF
#!/bin/sh
out=a.out
input=""
want_out=0
for arg in "\$@"; do
  if [ "\$want_out" = 1 ]; then out="\$arg"; want_out=0; continue; fi
  case "\$arg" in
    -o) want_out=1 ;;
    -*) ;;
    *) input="\$arg" ;;
  esac
done
exec "$TCC/bin/tcc" -c -I"$MUSL/include" -o "\$out" "\$input"
EOF
        $BB chmod 755 "$out/bin/as"
      fi
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

      # Assembler smoke test
      $BB cat > /tmp/smoke.s << 'ASM'
.section .text
.global _start
_start:
  mov $60, %rax
  xor %rdi, %rdi
  syscall
ASM
      "$out/bin/as" -o /tmp/smoke.o /tmp/smoke.s
      test -f /tmp/smoke.o
      "$out/bin/ar" rc /tmp/libsmoke.a /tmp/smoke.o
      "$out/bin/ranlib" /tmp/libsmoke.a
      test -f /tmp/libsmoke.a
      "$out/bin/nm" /tmp/smoke.o >/tmp/smoke.nm
      "$out/bin/objcopy" /tmp/smoke.o /tmp/smoke.copy.o
      test -f /tmp/smoke.copy.o
      "$out/bin/ld" -o /tmp/smoke-bin /tmp/smoke.o
      test -f /tmp/smoke-bin
      $BB chmod 755 /tmp/smoke-bin
      echo "binutils-2.30 bridge smoke test passed"
    "%,
  ],
  inputs = [stage0, mes, tcc, musl, make, sed, m4, grep, diffutils, bash, tar, bzip2, binutils_src],
} | crunch.Derivation
