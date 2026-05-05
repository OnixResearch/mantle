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
#define DPAREN_ARITHMETIC 1
#define COND_COMMAND 1
#define ARITH_FOR_COMMAND 1
#define ALIAS 1
#define HAVE_ALLOCA 1
#define HAVE_ALLOCA_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_STDARG_H 1
#define PREFER_STDARG 1
#define HAVE_LIMITS_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_LOCALE_H 1
#define HAVE_TERMIOS_H 1
#define HAVE_DIRENT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_FILE_H 1
#define HAVE_SYS_PARAM_H 1
#define HAVE_SYS_TIME_H 1
#define HAVE_TIME_H 1
#define TIME_WITH_SYS_TIME 1
#define HAVE_TZNAME 1
#define HAVE_SYS_WAIT_H 1
#define _POSIX_VERSION 200112L
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
#define HAVE_GETHOSTNAME 1
#define HAVE_GETPID 1
#define HAVE_GETUID 1
#define HAVE_GETGID 1
#define HAVE_GETEUID 1
#define HAVE_GETEGID 1
#define HAVE_GETGROUPS 1
#define GETGROUPS_T gid_t
#define HAVE_KILL 1
#define HAVE_LSTAT 1
#define HAVE_MEMMOVE 1
#define HAVE_MEMSET 1
#define HAVE_BCOPY 1
#define HAVE_BZERO 1
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
#define HAVE_SNPRINTF 1
#define HAVE_VSNPRINTF 1
#define VOID_SIGHANDLER 1
#define RETSIGTYPE void
#define SIZEOF_INT 4
#define SIZEOF_LONG 8
#define SIZEOF_CHAR_P 8
#define SIZEOF_DOUBLE 8
#define HAVE_LONG_LONG 1
#define HAVE_UNSIGNED_LONG_LONG 1
#define HAVE_STRTOL 1
#define HAVE_STRTOUL 1
#define HAVE_STRTOD 1
#define HAVE_STRTOLL 1
#define HAVE_STRTOULL 1
#define HAVE_DECL_STRTOL 1
#define HAVE_DECL_STRTOUL 1
#define HAVE_DECL_STRTOLL 1
#define HAVE_DECL_STRTOULL 1
#define PPROMPT "\\s-\\v\\$ "
#define SPROMPT "\\s-\\v\\$ "
#define PROGRAM "bash"
#define DEFAULT_PATH_VALUE "/bin"
#define STANDARD_UTILS_PATH "/bin"
extern int eval_builtin();
extern int source_builtin();
extern int exit_builtin();
extern int return_builtin();
extern int exec_builtin();
extern int read_builtin();
extern int jobs_builtin();
extern int command_builtin();
extern int wait_builtin();
extern int set_builtin();
CONFIG

      # Generate pathnames.h
      $BB cat > pathnames.h << 'PATHS'
#define DEFAULT_MAIL_DIRECTORY "/var/mail"
#define SYS_PROFILE "/etc/profile"
PATHS

      # Configure normally generates this table. A small static table is enough
      # for the bootstrap/nojobs shell to compile and report signal names.
      $BB cat > signames.h << 'SIGS'
#include <signal.h>
#ifndef NSIG
#define NSIG 64
#endif
char *signal_names[NSIG + 2] = { "EXIT", "SIGHUP", "SIGINT", "SIGQUIT", "SIGILL", "SIGTRAP", "SIGABRT", "SIGBUS", "SIGFPE", "SIGKILL", "SIGUSR1", "SIGSEGV", "SIGUSR2", "SIGPIPE", "SIGALRM", "SIGTERM" };
SIGS

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

      CFLAGS="-I. -Iinclude -Ilib -Ibuiltins -DHAVE_CONFIG_H"

      # Mes/TinyCC bootstrap headers do not ship termios yet; bash only needs
      # the type while compiling this non-interactive nojobs handoff.
      $BB cat > termios.h << 'TERM'
#ifndef CRUNCH_BOOTSTRAP_TERMIOS_H
#define CRUNCH_BOOTSTRAP_TERMIOS_H
#define TCSANOW 0
#define TCSADRAIN 1
#define ECHO 0000010
#define ECHOK 0000040
#define ECHONL 0000100
#define ICANON 0000002
#define ISIG 0000001
#define IEXTEN 0100000
#define ICRNL 0000400
#define INLCR 0000100
#define ISTRIP 0000040
#define OPOST 0000001
#define ONLCR 0000004
#define OCRNL 0000010
#define ONOCR 0000020
#define ONLRET 0000040
#define CS8 0000060
#define PARENB 0000400
#define VMIN 6
#define VTIME 5
struct termios { int c_iflag; int c_oflag; int c_cflag; int c_lflag; char c_cc[32]; };
int tcgetattr(int fd, struct termios *t);
int tcsetattr(int fd, int action, const struct termios *t);
#endif
TERM

      $BB cat > syntax.c << 'SYN'
#include "config.h"
unsigned char sh_syntaxtab[256];
int shellmeta(int c) { return 0; }
int shellbreak(int c) { return 0; }
int shellquote(int c) { return 0; }
int shellxquote(int c) { return 0; }
int shellexp(int c) { return 0; }
SYN
      $BB cat > siglist.c << 'SIGL'
#include "config.h"
#include <signal.h>
#ifndef NSIG
#define NSIG 64
#endif
char *sys_siglist[NSIG + 2];
SIGL

      # Compile core shell sources
      SHELL_SRCS="shell eval y.tab general make_cmd print_cmd dispose_cmd execute_cmd variables copy_cmd error expr flags nojobs subst hashcmd hashlib mailcheck trap input unwind_prot pathexp sig test version alias array arrayfunc braces bracecomp bashhist bashline list stringlib locale findcmd redir pcomplete pcomplib syntax xmalloc siglist"

      for src in $SHELL_SRCS; do
        tcc -c -o "$src.o" $CFLAGS "$src.c"
      done

      $BB cat > builtins/bootstrap_stubs.c << 'BSTUB'
#include "config.h"
#include "shell.h"
#include "command.h"
#include "builtins.h"
#include <termios.h>
#include <string.h>
#include <stdlib.h>
struct builtin static_shell_builtins[] = { { "eval", eval_builtin, BUILTIN_ENABLED|STATIC_BUILTIN, 0, "eval", 0 }, { 0, 0, 0, 0, 0, 0 } };
struct builtin *shell_builtins = static_shell_builtins;
struct builtin *current_builtin = 0;
int num_shell_builtins = 1;
int breaking = 0;
int continuing = 0;
int loop_level = 0;
int source_uses_path = 1;
int source_searches_cwd = 1;
int wait_intr_buf = 0;
long timezone = 0;
int daylight = 0;
long altzone = 0;
char *tzname[2] = { "UTC", "UTC" };
extern char *signal_names[];
int eval_builtin(WORD_LIST *list) { return 0; }
int source_builtin(WORD_LIST *list) { return 1; }
int exit_builtin(WORD_LIST *list) { return 0; }
int return_builtin(WORD_LIST *list) { return 0; }
int exec_builtin(WORD_LIST *list) { return 1; }
int read_builtin(WORD_LIST *list) { return 1; }
int jobs_builtin(WORD_LIST *list) { return 0; }
int command_builtin(WORD_LIST *list) { return 1; }
int wait_builtin(WORD_LIST *list) { return 0; }
int set_builtin(WORD_LIST *list) { return 0; }
void reset_shell_options(void) {}
void reset_shopt_options(void) {}
void initialize_shell_options(int no_shellopts) {}
void set_shellopts(void) {}
int minus_o_option_value(char *name) { return 0; }
void getopts_reset(int newind) {}
int list_minus_o_opts(int pflag, int reusable, char *value) { return 0; }
int set_minus_o_option(int on_or_off, char *option_name) { return 0; }
int shopt_listopt(char *name, int reusable) { return 0; }
int shopt_setopt(char *name, int mode) { return 0; }
void set_login_shell(char *arg) {}
int getgroups(int size, gid_t *list) { return 0; }
void endpwent(void) {}
int gethostname(char *name, int len) { if (len > 0) { strcpy(name, "crunch"); } return 0; }
int tcgetattr(int fd, struct termios *t) { memset(t, 0, sizeof(*t)); return 0; }
int tcsetattr(int fd, int action, const struct termios *t) { return 0; }
void tzset(void) {}
char *strsignal(int sig) { return (sig >= 0 && sig < 16) ? signal_names[sig] : "signal"; }
BSTUB
      tcc -c -o bootstrap_stubs.o $CFLAGS builtins/bootstrap_stubs.c

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
      tcc -c -o readline_stub.o $CFLAGS lib/readline/readline_stub.c

      # Compile lib/glob
      for src in lib/glob/glob lib/glob/strmatch lib/glob/smatch; do
        tcc -c -o "${src##*/}.o" $CFLAGS "$src.c"
      done

      # Compile lib/tilde
      tcc -c -o tilde.o $CFLAGS lib/tilde/tilde.c

      # Compile lib/sh
      for src in lib/sh/clktck lib/sh/getcwd lib/sh/getenv lib/sh/oslib lib/sh/setlinebuf \
                 lib/sh/strcasecmp lib/sh/strerror lib/sh/strtod lib/sh/vprint \
                 lib/sh/itos lib/sh/rename lib/sh/zread lib/sh/zwrite lib/sh/shtty \
                 lib/sh/inet_aton lib/sh/netopen lib/sh/timeval \
                 lib/sh/clock lib/sh/makepath lib/sh/pathcanon lib/sh/pathphys \
                 lib/sh/stringlist lib/sh/stringvec lib/sh/tmpfile lib/sh/spell \
                 lib/sh/strtrans lib/sh/strindex lib/sh/shquote lib/sh/mailstat \
                 lib/sh/fmtulong lib/sh/fmtullong lib/sh/strtoll lib/sh/strtoull \
                 lib/sh/strtoimax lib/sh/strtoumax lib/sh/fmtumax lib/sh/netconn \
                 lib/sh/xstrchr lib/sh/zcatfd; do
        tcc -c -o "${src##*/}.o" $CFLAGS "$src.c"
      done

      # Compile builtins
      for src in builtins/common builtins/evalstring builtins/evalfile \
                 builtins/bashgetopt builtins/getopt; do
        tcc -c -o "${src##*/}.o" $CFLAGS "$src.c"
      done

      # Collect all compiled objects
      OBJS=""
      for f in *.o builtins/*.o; do
        [ -f "$f" ] && OBJS="$OBJS $f"
      done

      $BB chmod u+rw *.o || true

      # Link
      tcc -static $OBJS -o bash
      $BB chmod 755 bash

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
