# Diagnostic: rebuild Mes libc with TinyCC x86_64 stdarg and rerun runtime format probe.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-0.27.1-src",
} in

{
  name = "diag-tcc27-runtime-patched-stdarg-libc",
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
      TCC26=$(find_input tinycc-0.9.26)
      STAGE0=$(find_input stage0-posix)
      MES_SRC=$(find_input mes-0.27.1-src)
      export PATH="$TCC/bin:$STAGE0/bin:$PATH"
      MES_ARCH=x86_64
      WORK=/tmp/tcc27-runtime-patched-stdarg-libc
      MES_BUILD="$WORK/mes-0.27.1"
      LIBDIR="$WORK/lib"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$LIBDIR/tcc"
      $BB cp -r "$MES_SRC" "$MES_BUILD"
      $BB chmod -R u+w "$MES_BUILD"
      cd "$MES_BUILD"
      $BB mkdir -p include/mes
      $BB cat > include/mes/config.h <<'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF
      # Use TinyCC's x86_64 SysV va_list ABI for objects compiled by TinyCC.
      $BB cat > include/stdarg.h <<'EOF'
#ifndef __MES_STDARG_H
#define __MES_STDARG_H 1
#if defined(__TINYC__) && defined(__x86_64__)
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
typedef __va_list_struct va_list[1];
void __va_start(__va_list_struct *ap, void *fp);
void *__va_arg(__va_list_struct *ap, int arg_type, int size, int align);
#define va_start(ap, last) __va_start(ap, __builtin_frame_address(0))
#define va_arg(ap, type) (*(type *)(__va_arg(ap, __builtin_va_arg_types(type), sizeof(type), __alignof__(type))))
#define va_arg8(ap, type) va_arg(ap, type)
#define va_copy(dest, src) (*(dest) = *(src))
#define va_end(ap)
#else
#include_next <stdarg.h>
#define va_arg8(ap, type) va_arg(ap, type)
#endif
#endif
EOF

      create_unified_libc() {
        cd "$MES_BUILD/lib"
        $BB cat \
          ctype/isalnum.c ctype/isalpha.c ctype/isascii.c ctype/iscntrl.c \
          ctype/isdigit.c ctype/isgraph.c ctype/islower.c ctype/isnumber.c \
          ctype/isprint.c ctype/ispunct.c ctype/isspace.c ctype/isupper.c \
          ctype/isxdigit.c ctype/tolower.c ctype/toupper.c \
          dirent/closedir.c dirent/__getdirentries.c dirent/opendir.c \
          linux/readdir.c linux/access.c linux/brk.c linux/chdir.c \
          linux/chmod.c linux/clock_gettime.c linux/close.c linux/dup2.c \
          linux/dup.c linux/execve.c linux/fcntl.c linux/fork.c linux/fsync.c \
          linux/fstat.c linux/_getcwd.c linux/getdents.c linux/getegid.c \
          linux/geteuid.c linux/getgid.c linux/getpid.c linux/getppid.c \
          linux/getrusage.c linux/gettimeofday.c linux/getuid.c linux/ioctl.c \
          linux/ioctl3.c linux/kill.c linux/link.c linux/lseek.c linux/lstat.c \
          linux/malloc.c linux/mkdir.c linux/mknod.c linux/nanosleep.c \
          linux/_open3.c linux/pipe.c linux/_read.c linux/readlink.c \
          linux/rename.c linux/rmdir.c linux/setgid.c linux/settimer.c \
          linux/setuid.c linux/signal.c linux/sigprogmask.c linux/symlink.c \
          linux/stat.c linux/time.c linux/unlink.c linux/waitpid.c linux/wait4.c \
          linux/${MES_ARCH}-mes-gcc/_exit.c linux/${MES_ARCH}-mes-gcc/syscall.c \
          linux/${MES_ARCH}-mes-gcc/_write.c \
          math/ceil.c math/fabs.c math/floor.c \
          mes/abtod.c mes/abtol.c mes/__assert_fail.c mes/assert_msg.c \
          mes/__buffered_read.c mes/__init_io.c mes/cast.c mes/dtoab.c \
          mes/eputc.c mes/eputs.c mes/fdgetc.c mes/fdgets.c mes/fdputc.c \
          mes/fdputs.c mes/fdungetc.c mes/globals.c mes/itoa.c mes/ltoab.c \
          mes/ltoa.c mes/__mes_debug.c mes/mes_open.c mes/ntoab.c mes/oputc.c \
          mes/oputs.c mes/search-path.c mes/ultoa.c mes/utoa.c \
          posix/alarm.c posix/buffered-read.c posix/execl.c posix/execlp.c \
          posix/execv.c posix/execvp.c posix/getcwd.c posix/getenv.c \
          posix/isatty.c posix/mktemp.c posix/open.c posix/pathconf.c \
          posix/raise.c posix/sbrk.c posix/setenv.c posix/sleep.c \
          posix/unsetenv.c posix/wait.c posix/write.c \
          stdio/clearerr.c stdio/fclose.c stdio/fdopen.c stdio/feof.c \
          stdio/ferror.c stdio/fflush.c stdio/fgetc.c stdio/fgets.c \
          stdio/fileno.c stdio/fopen.c stdio/fprintf.c stdio/fputc.c \
          stdio/fputs.c stdio/fread.c stdio/freopen.c stdio/fscanf.c \
          stdio/fseek.c stdio/ftell.c stdio/fwrite.c stdio/getc.c \
          stdio/getchar.c stdio/perror.c stdio/printf.c stdio/putc.c \
          stdio/putchar.c stdio/remove.c stdio/snprintf.c stdio/sprintf.c \
          stdio/sscanf.c stdio/ungetc.c stdio/vfprintf.c stdio/vfscanf.c \
          stdio/vprintf.c stdio/vsnprintf.c stdio/vsprintf.c stdio/vsscanf.c \
          stdlib/abort.c stdlib/abs.c stdlib/alloca.c stdlib/atexit.c \
          stdlib/atof.c stdlib/atoi.c stdlib/atol.c stdlib/calloc.c \
          stdlib/__exit.c stdlib/exit.c stdlib/free.c stdlib/mbstowcs.c \
          stdlib/puts.c stdlib/qsort.c stdlib/realloc.c stdlib/strtod.c \
          stdlib/strtof.c stdlib/strtol.c stdlib/strtold.c stdlib/strtoll.c \
          stdlib/strtoul.c stdlib/strtoull.c \
          string/bcmp.c string/bcopy.c string/bzero.c string/index.c \
          string/memchr.c string/memcmp.c string/memcpy.c string/memmem.c \
          string/memmove.c string/memset.c string/rindex.c string/strcat.c \
          string/strchr.c string/strcmp.c string/strcpy.c string/strcspn.c \
          string/strdup.c string/strerror.c string/strlen.c string/strlwr.c \
          string/strncat.c string/strncmp.c string/strncpy.c string/strpbrk.c \
          string/strrchr.c string/strspn.c string/strstr.c string/strupr.c \
          stub/atan2.c stub/bsearch.c stub/chown.c stub/__cleanup.c \
          stub/cos.c stub/ctime.c stub/exp.c stub/fpurge.c stub/freadahead.c \
          stub/frexp.c stub/getgrgid.c stub/getgrnam.c stub/getlogin.c \
          stub/getpgid.c stub/getpgrp.c stub/getpwnam.c stub/getpwuid.c \
          stub/gmtime.c stub/ldexp.c stub/localtime.c stub/log.c stub/mktime.c \
          stub/modf.c stub/mprotect.c stub/pclose.c stub/popen.c stub/pow.c \
          stub/putenv.c stub/rand.c stub/realpath.c stub/rewind.c \
          stub/setbuf.c stub/setgrent.c stub/setlocale.c stub/setvbuf.c \
          stub/sigaction.c stub/sigaddset.c stub/sigblock.c stub/sigdelset.c \
          stub/sigemptyset.c stub/sigsetmask.c stub/sin.c stub/sys_siglist.c \
          stub/system.c stub/sqrt.c stub/strftime.c stub/times.c stub/ttyname.c \
          stub/umask.c stub/utime.c ${MES_ARCH}-mes-gcc/setjmp.c \
          > ../unified-libc.c
        cd "$MES_BUILD"
      }

      create_unified_libc

      $BB cat > va_list.c <<'EOF'
#if defined __x86_64__
extern void *memset(void *s, int c, unsigned long n);
extern void abort(void);
enum __va_arg_type { __va_gen_reg, __va_float_reg, __va_stack };
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
void __va_start(__va_list_struct *ap, void *fp) {
    memset(ap, 0, sizeof(__va_list_struct));
    *ap = *(__va_list_struct *)((char *)fp - 16);
    ap->overflow_arg_area = (char *)fp + ap->overflow_offset;
    ap->reg_save_area = (char *)fp - 176 - 16;
}
void *__va_arg(__va_list_struct *ap, enum __va_arg_type arg_type, int size, int align) {
    size = (size + 7) & ~7;
    align = (align + 7) & ~7;
    switch (arg_type) {
    case __va_gen_reg:
        if (ap->gp_offset + size <= 48) { ap->gp_offset += size; return ap->reg_save_area + ap->gp_offset - size; }
        goto use_overflow_area;
    case __va_float_reg:
        if (ap->fp_offset < 128 + 48) { ap->fp_offset += 16; return ap->reg_save_area + ap->fp_offset - 16; }
        size = 8;
        goto use_overflow_area;
    case __va_stack:
    use_overflow_area:
        ap->overflow_arg_area += size;
        ap->overflow_arg_area = (char*)((long long)(ap->overflow_arg_area + align - 1) & -align);
        return ap->overflow_arg_area - size;
    default:
        abort();
    }
}
#endif
EOF
      echo "diag-patched-libc: compile va_list" >&2
      "$TCC26/bin/tcc" -c -D HAVE_CONFIG_H=1 -I include -I include/linux/${MES_ARCH} -o va_list.o va_list.c
      echo "diag-patched-libc: compile unified libc" >&2
      "$TCC26/bin/tcc" -w -c -D HAVE_CONFIG_H=1 -I include -I include/linux/${MES_ARCH} -o unified-libc.o unified-libc.c
      "$TCC26/bin/tcc" -ar cr "$LIBDIR/libc.a" va_list.o unified-libc.o

      cd "$WORK"
      $BB cat > helper.c <<'EOF'
      typedef unsigned long size_t;
      extern int strcmp(const char *, const char *);
      extern int sprintf(char *, const char *, ...);
      extern int puts(const char *);
      extern void *memset(void *, int, size_t);
      static int global_counter = 41;
      static const char *global_words[] = { "alpha", "beta", "gamma", 0 };
      int helper_value(const char *needle) {
        int i; int score = 0; char buf[64]; memset(buf, 0, sizeof(buf));
        for (i = 0; global_words[i]; i++) if (!strcmp(global_words[i], needle)) score += i + 1;
        sprintf(buf, "score=%d", score + global_counter);
        if (strcmp(buf, "score=43")) { puts(buf); return 10; }
        puts(buf); return 0;
      }
EOF
      $BB cat > main.c <<'EOF'
      extern int helper_value(const char *);
      extern int printf(const char *, ...);
      extern int puts(const char *);
      static int add3(int x) { return x + 3; }
      static int (*global_fp)(int) = add3;
      static const char banner[] = "diag-tcc27-runtime-patched-stdarg-libc";
      int main(int argc, char **argv) {
        int v; (void)argc; (void)argv; puts(banner); v = global_fp(39);
        if (v != 42) return 20;
        if (helper_value("beta")) return 21;
        printf("value=%d\n", v);
        return 0;
      }
EOF
      tcc -c helper.c
      tcc -c main.c
      $BB chmod 644 helper.o main.o
      echo "diag-patched-libc: link explicit static executable" >&2
      tcc -nostdlib -static -o diag         "$TCC/lib/mes/crt1.o" main.o helper.o "$LIBDIR/libc.a" "$TCC/lib/mes/tcc/libtcc1.a"
      $BB chmod 755 diag
      ./diag > probe.stdout 2>probe.stderr; rc=$?; echo "$rc" > probe.rc
      $BB mkdir -p "$out/bin" "$out/objects" "$out/src"
      $BB cp diag "$out/bin/diag-tcc27-runtime-patched-stdarg-libc"
      $BB cp helper.o main.o "$out/objects/"
      $BB cp helper.c main.c probe.stdout probe.stderr probe.rc "$out/src/"
      test "$rc" = 0
    "%,
  ],
  inputs = [stage0, tcc, tcc26, mes_src],
} | crunch.Derivation
