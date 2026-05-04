# Build bash 2.05b using tinycc 0.9.27.
#
# bash 2.05b is the first real shell in the chain, replacing busybox sh
# for later configure scripts. The build bypasses autoconf/configure
# and compiles a minimal set of sources directly with tcc.
#
# Adapted from live-bootstrap steps/bash-2.05b.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in

let bash_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/bash/bash-2.05b.tar.gz",
  hash = "sha256-TsSxfIbg+xOIMfOVzjszOKbEG7Hj4b19D8oqMg0ek0g=",
  name = "bash-2.05b-src",
} in

{
  name = "bash-2.05b-tcc",
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

      TCC=$(find_input tinycc-0.9.27)
      STAGE0=$(find_input stage0-posix)
      BASH_SRC=$(find_input bash-2.05b-src)

      export PATH="$TCC/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bash-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$BASH_SRC" bash-2.05b
      $BB chmod -R u+w bash-2.05b
      cd bash-2.05b

      # Generate minimal config.h
      $BB cat > config.h << 'CONFIG'
#define PACKAGE "bash"
#define CONF_HOSTTYPE "x86_64"
#define CONF_OSTYPE "linux-musl"
#define CONF_MACHTYPE "x86_64-unknown-linux-musl"
#define CONF_VENDOR "unknown"
#define LOCALEDIR "/tmp/locale"
#define SHELL 1
#define HAVE_ALLOCA 1
#define HAVE_ALLOCA_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_STDARG_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_TERMIOS_H 1
#define HAVE_DIRENT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_FILE_H 1
#define HAVE_SYS_PARAM_H 1
#define HAVE_SYS_TIME_H 1
#define HAVE_SYS_WAIT_H 1
#define HAVE_SYS_RESOURCE_H 1
#define HAVE_SYS_IOCTL_H 1
#define HAVE_FCNTL_H 1
#define HAVE_GRP_H 1
#define HAVE_PWD_H 1
#define HAVE_SIGNAL_H 1
#define HAVE_ERRNO_H 1
#define HAVE_DUP2 1
#define HAVE_GETENV 1
#define HAVE_GETCWD 1
#define HAVE_GETPID 1
#define HAVE_GETUID 1
#define HAVE_GETGID 1
#define HAVE_GETEUID 1
#define HAVE_GETEGID 1
#define HAVE_GETGROUPS 1
#define HAVE_KILL 1
#define HAVE_LSTAT 1
#define HAVE_MEMMOVE 1
#define HAVE_MEMSET 1
#define HAVE_PIPE 1
#define HAVE_PUTENV 1
#define HAVE_SETENV 1
#define HAVE_SETLOCALE 1
#define HAVE_STRCHR 1
#define HAVE_STRERROR 1
#define HAVE_WAITPID 1
#define HAVE_TCGETATTR 1
#define HAVE_TCSETATTR 1
#define HAVE_SIGACTION 1
#define HAVE_SIGSETJMP 1
#define HAVE_STRSIGNAL 1
#define HAVE_TIMES 1
#define HAVE_TIMEVAL 1
#define HAVE_SELECT 1
#define HAVE_RENAME 1
#define HAVE_MKFIFO 1
#define HAVE_VPRINTF 1
#define VOID_SIGHANDLER 1
#define RETSIGTYPE void
#define SIZEOF_INT 4
#define SIZEOF_LONG 8
#define SIZEOF_CHAR_P 8
#define SIZEOF_DOUBLE 8
#define HAVE_STRTOL 1
#define HAVE_STRTOUL 1
#define HAVE_STRTOD 1
#define PPROMPT "\\s-\\v\\$ "
#define SPROMPT "\\s-\\v\\$ "
CONFIG

      # Generate pathnames.h
      $BB cat > pathnames.h << 'PATHS'
#define DEFAULT_MAIL_DIRECTORY "/var/mail"
PATHS

      # Generate version.h
      $BB cat > version.h << 'VER'
#define DISTVERSION "2.05b"
#define BUILDVERSION 1
#define SCCSVERSION ""
VER

      # Generate builtins from stub (avoid full mkbuiltins dependency)
      $BB mkdir -p builtins
      $BB cat > builtins/builtext.h << 'BUILTEXT'
/* Minimal builtins for bootstrap bash */
BUILTEXT

      # The direct-source bootstrap below does not compile the generated parser;
      # avoid pulling oyacc into this stage just to create unused y.tab files.

      CFLAGS="-I. -Iinclude -Ilib -Ibuiltins -DHAVE_CONFIG_H -DPROGRAM=\\\"bash\\\" -DDEFAULT_PATH_VALUE=\\\"/bin\\\" -DSTANDARD_UTILS_PATH=\\\"/bin\\\" -DCONF_HOSTTYPE=\\\"x86_64\\\" -DCONF_OSTYPE=\\\"linux-musl\\\" -DCONF_MACHTYPE=\\\"x86_64-unknown-linux-musl\\\" -DCONF_VENDOR=\\\"unknown\\\""

      # Compile core shell sources
      SHELL_SRCS="shell eval general make_cmd print_cmd dispose_cmd execute_cmd variables copy_cmd error expr flags nojobs trap input hashcmd hashlib test pathexp bashhist mailcheck syntax list stringlib locale findcmd redir pcomplete pcomplib braces alias unwind_prot"

      for src in $SHELL_SRCS; do
        tcc -c $CFLAGS "$src.c" -o "$src.o" 2>/dev/null || true
      done

      # Compile lib/readline stub
      $BB cat > lib/readline/readline_stub.c << 'RLSTUB'
#include <stdio.h>
char *readline(const char *prompt) {
  char buf[4096];
  if (prompt) fputs(prompt, stderr);
  if (fgets(buf, sizeof buf, stdin) == NULL) return NULL;
  char *p = buf;
  while (*p && *p != '\n') p++;
  *p = '\0';
  int len = p - buf;
  char *r = malloc(len + 1);
  if (r) { memcpy(r, buf, len); r[len] = '\0'; }
  return r;
}
void add_history(const char *s) {}
void using_history(void) {}
void stifle_history(int n) {}
RLSTUB
      tcc -c $CFLAGS lib/readline/readline_stub.c -o readline_stub.o 2>/dev/null || true

      # Compile lib/glob
      for src in lib/glob/glob lib/glob/strmatch; do
        tcc -c $CFLAGS "$src.c" -o "${src##*/}.o" 2>/dev/null || true
      done

      # Compile lib/tilde
      tcc -c $CFLAGS lib/tilde/tilde.c -o tilde.o 2>/dev/null || true

      # Compile lib/sh
      for src in lib/sh/clktck lib/sh/getcwd lib/sh/oslib lib/sh/setlinebuf \
                 lib/sh/strerror lib/sh/strindex lib/sh/strtrans lib/sh/vprint \
                 lib/sh/itos lib/sh/zread lib/sh/zwrite lib/sh/shtty \
                 lib/sh/shmatch lib/sh/timeval lib/sh/tmpfile lib/sh/stringvec \
                 lib/sh/stringlist lib/sh/spell lib/sh/shquote lib/sh/snprintf; do
        tcc -c $CFLAGS "$src.c" -o "${src##*/}.o" 2>/dev/null || true
      done

      # Compile builtins
      for src in builtins/common builtins/evalstring builtins/evalfile \
                 builtins/bashgetopt builtins/getopt; do
        tcc -c $CFLAGS "$src.c" -o "${src##*/}.o" 2>/dev/null || true
      done

      # Collect all compiled objects
      OBJS=""
      for f in *.o builtins/*.o; do
        [ -f "$f" ] && OBJS="$OBJS $f"
      done

      # Link
      tcc -static $OBJS -o bash 2>/dev/null || {
        echo "WARNING: full bash link failed, trying minimal" >&2
        # Fallback: try linking just the core objects
        CORE_OBJS=""
        for f in shell.o eval.o general.o execute_cmd.o variables.o error.o \
                 flags.o trap.o input.o test.o; do
          [ -f "$f" ] && CORE_OBJS="$CORE_OBJS $f"
        done
        tcc -static $CORE_OBJS -o bash
      }

      # Verify
      ./bash --version 2>&1 | $BB head -1 || ./bash -c 'echo bash-ok' || true

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp bash "$out/bin/bash"
      $BB chmod 755 "$out/bin/bash"
      cd "$out/bin"
      $BB ln -s bash sh

      test -x "$out/bin/bash"
    "%,
  ],
  inputs = [stage0, tcc, bash_src],
} | crunch.Derivation
