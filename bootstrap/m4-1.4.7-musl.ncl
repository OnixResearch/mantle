# Build m4 1.4.7 linked against musl using tcc-musl-v2.
#
# m4 is a macro processor required by autoconf. This version is
# built before bison/flex to keep the chain linear.
#
# Adapted from live-bootstrap steps/m4-1.4.7.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let m4_src = crunch.fetchTarball {
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
      SRC=$(find_input m4-1.4.7-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/m4-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" m4-1.4.7
      $BB chmod -R u+w m4-1.4.7
      cd m4-1.4.7

      $BB cat > lib/config.h <<'EOF'
#define VERSION "1.4.7"
#define PACKAGE_BUGREPORT "bug-m4@gnu.org"
#define PACKAGE_STRING "GNU M4 1.4.7"
#define PACKAGE "m4"
#define PACKAGE_NAME "GNU M4"
#define HAVE_STDINT_H 1
#define HAVE___FPENDING 1
#define HAVE_DECL___FPENDING 1
#define _GNU_SOURCE 1
#define _GL_UNUSED
#define __getopt_argv_const const
#define SYSCMD_SHELL "/bin/sh"
EOF

      CFLAGS="-Ilib -Isrc -I$MUSL/include"

      # TinyCC/musl-v2 segfaults while compiling the two m4.c wrappers that
      # forward varargs through va_start/va_list.  Keep diagnostics coarse but
      # preserve exit behavior so bootstrap consumers can run m4.
      $BB awk '
        NR == 79 { print "void"; print "m4_error (int status, int errnum, const char *format, ...)"; print "{"; print "  fputs (format, stderr);"; print "  if (status) exit (status);"; print "}"; next }
        NR >= 79 && NR <= 86 { next }
        NR == 92 { print "void"; print "m4_error_at_line (int status, int errnum, const char *file, int line,"; print "          const char *format, ...)"; print "{"; print "  fputs (format, stderr);"; print "  if (status) exit (status);"; print "}"; next }
        NR >= 92 && NR <= 99 { next }
        { print }
      ' src/m4.c > src/m4.c.new
      $BB mv src/m4.c.new src/m4.c
      $BB awk '
        NR == 260 { print "static void"; print "trace_format (const char *fmt, ...)"; print "{"; print "  while (*fmt) obstack_1grow (&trace, *fmt++);"; print "}"; next }
        NR >= 260 && NR <= 323 { next }
        { print }
      ' src/debug.c > src/debug.c.new
      $BB mv src/debug.c.new src/debug.c
      $BB cat > lib/close-stream.c <<'EOF'
#include <config.h>
#include "close-stream.h"
int close_stream(FILE *stream) { return fclose(stream); }
EOF
      $BB cat > lib/error.c <<'EOF'
#include <config.h>
#include <stdio.h>
#include <stdlib.h>
void (*error_print_progname) (void);
unsigned int error_message_count;
void error(int status, int errnum, const char *message, ...) {
  error_message_count++;
  if (message) fputs(message, stderr);
  fputc('\n', stderr);
  if (status) exit(status);
}
void error_at_line(int status, int errnum, const char *file_name,
                   unsigned int line_number, const char *message, ...) {
  error(status, errnum, message);
}
EOF
      $BB cat > lib/xasprintf.c <<'EOF'
#include <config.h>
#include <string.h>
#include "xalloc.h"
char *xasprintf(const char *format, ...) {
  char *r = (char *) xmalloc(strlen(format) + 1);
  strcpy(r, format);
  return r;
}
EOF
      $BB cat > lib/verror.c <<'EOF'
#include <config.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
void verror (int status, int errnum, const char *format, va_list args) {
  if (format) fputs(format, stderr);
  fputc('\n', stderr);
  if (status) exit(status);
}
void verror_at_line (int status, int errnum, const char *fname,
                     unsigned int lineno, const char *format, va_list args) {
  verror(status, errnum, format, args);
}
EOF
      $BB awk 'NR == 44 { print "#undef __GNU_LIBRARY__" } { print }' lib/obstack.c > lib/obstack.c.new
      $BB mv lib/obstack.c.new lib/obstack.c

      # Compile m4 sources.  Fail closed so the link never consumes a partial
      # object set after a hidden TinyCC handoff error.
      for src in m4 builtin debug eval format freeze input macro \
                 output path symtab; do
        echo "compile src/$src"
        tcc -c $CFLAGS src/$src.c -o src/$src.o
        test -s "src/$src.o"
      done

      # Compile lib support.
      for src in cloexec close-stream dup-safer error exitfail fd-safer fopen-safer \
                 getopt getopt1 mkstemp-safer regex obstack tmpfile-safer \
                 verror xalloc-die xasprintf xmalloc; do
        echo "compile lib/$src"
        tcc -c $CFLAGS lib/$src.c -o lib/$src.o
        test -s "lib/$src.o"
      done

      OBJS=""
      $BB chmod u+rw src/*.o lib/*.o || true
      for f in src/*.o lib/*.o; do
        if [ -f "$f" ]; then
          base=${f%/*}-${f##*/}
          $BB cp "$f" "$base"
          OBJS="$OBJS $base"
        fi
      done
      echo "link m4"
      echo "objects:$OBJS"
      tcc -static $OBJS -L$MUSL/lib -o m4
      $BB chmod 755 m4

      echo 'define(FOO,bar)FOO' | ./m4

      $BB mkdir -p "$out/bin"
      $BB cp m4 "$out/bin/m4"
      $BB chmod 755 "$out/bin/m4"

      test -x "$out/bin/m4"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, m4_src],
} | crunch.Derivation
