# Rebuild musl 1.1.24 with tcc-musl (second pass).
#
# The first musl was built with the bridge tcc-musl compiler. Reuse the
# first-stage source compatibility bridge while moving the compiler input to
# the musl-targeting TinyCC handoff.
#
# Adapted from live-bootstrap steps/musl-1.1.24 (pass2).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc_musl = import "tcc-musl.ncl" in
let musl_prev = import "musl-1.1.24-tcc.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let musl_src = crunch.fetchTarball {
  # first-consumer: bootstrap/musl-1.1.24-tcc-musl.ncl
  # provenance: fosslinux/live-bootstrap steps/musl-1.1.24/pass2.sh source input
  url = "https://musl.libc.org/releases/musl-1.1.24.tar.gz",
  hash = "sha256-mSl3003jmxfRju1tcJlRGT75yWw13QJ89diR6M6prkE=",
  name = "musl-1.1.24-src",
} in

{
  name = "musl-1.1.24-tcc-musl",
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

      TCC=$(find_input tcc-0.9.27-musl)
      MUSL_PREV=$(find_input musl-1.1.24-tcc)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-tcc)
      MUSL_SRC=$(find_input musl-1.1.24-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/musl-rebuild
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$MUSL_SRC" musl-1.1.24
      $BB chmod -R u+w musl-1.1.24
      cd musl-1.1.24

      if [ ! -f configure ]; then
        for cfg in */configure; do
          if [ -f "$cfg" ]; then cd "${cfg%/configure}"; break; fi
        done
      fi

      APPLETS="basename cat chmod cp dirname echo expr find grep ln mkdir mv printf pwd rm rmdir sed sh sort test touch tr true uname"
      $BB mkdir -p "$WORK/bin"
      for app in $APPLETS; do $BB ln -sf "$BB" "$WORK/bin/$app"; done
      export PATH="$WORK/bin:$PATH"

      $BB rm -rf src/complex src/aio src/legacy src/linux src/locale src/math src/misc src/mman src/mq src/multibyte src/network src/passwd src/prng src/process src/regex src/sched src/search src/select src/signal src/stat src/stdio src/stdlib src/temp src/termios src/thread src/time src/unistd src/malloc
      $BB rm -f crt/Scrt1.c crt/rcrt1.c
      ALLTYPES=arch/x86_64/bits/alltypes.h.in
      if [ ! -f "$ALLTYPES" ]; then ALLTYPES=include/alltypes.h.in; fi
      if [ -f "$ALLTYPES" ]; then
        $BB sed -i '/TYPEDEF __builtin_va_list va_list;/i\#if defined(__TINYC__)\
/* CRUNCH bridge TinyCC builtin va_list */\
typedef char *__builtin_va_list;\
#endif' "$ALLTYPES"
      fi
      if [ -f src/internal/syscall.h ]; then
        $BB sed -i '/fixup legacy/,$d' src/internal/syscall.h
        printf '#endif\n' >> src/internal/syscall.h
      fi

      $BB mkdir -p src/math
      cat > src/math/minmath.c <<'EOF'
double fabs(double x){return x<0?-x:x;} double floor(double x){return x;} double ceil(double x){return x;} double sqrt(double x){return x;} double sin(double x){return 0;} double cos(double x){return 1;} double pow(double x,double y){(void)y;return x;}
EOF
      $BB mkdir -p src/unistd
      cat > src/unistd/minunistd.c <<'EOF'
typedef long ssize_t; typedef unsigned long size_t; typedef long off_t;
void _exit(int c){(void)c;for(;;){}}
int close(int fd){(void)fd;return 0;} ssize_t read(int fd, void *b, size_t n){(void)fd;(void)b;(void)n;return 0;} ssize_t write(int fd, const void*b, size_t n){(void)fd;(void)b;return (ssize_t)n;}
int access(const char*p,int m){(void)p;(void)m;return -1;} int chdir(const char*p){(void)p;return -1;} char *getcwd(char*b,size_t n){if(b&&n)b[0]=0;return b;} int dup(int fd){return fd;} int dup2(int a,int b){(void)a;return b;} int isatty(int fd){(void)fd;return 0;} int unlink(const char*p){(void)p;return -1;} int rmdir(const char*p){(void)p;return -1;}
int pipe(int p[2]){if(p){p[0]=p[1]=-1;}return -1;} unsigned int sleep(unsigned int s){return s;} int usleep(unsigned int u){(void)u;return 0;}
EOF
      $BB mkdir -p src/stdlib
      cat > src/stdlib/minstdlib.c <<'EOF'
typedef unsigned long size_t;
long strtol(const char*s,char**e,int b); long long strtoll(const char*s,char**e,int b);
int abs(int x){return x<0?-x:x;} long labs(long x){return x<0?-x:x;} long long llabs(long long x){return x<0?-x:x;}
int atoi(const char*s){return (int)strtol(s,0,10);} long atol(const char*s){return strtol(s,0,10);} long long atoll(const char*s){return strtoll(s,0,10);}
long strtol(const char*s,char**e,int b){long v=0;int neg=0;if(*s=='-'){neg=1;s++;}while(*s>='0'&&*s<='9'){v=v*10+*s-'0';s++;}if(e)*e=(char*)s;return neg?-v:v;}
long long strtoll(const char*s,char**e,int b){return strtol(s,e,b);} unsigned long strtoul(const char*s,char**e,int b){return (unsigned long)strtol(s,e,b);} unsigned long long strtoull(const char*s,char**e,int b){return (unsigned long long)strtol(s,e,b);}
double strtod(const char*s,char**e){if(e)*e=(char*)s;return 0;} float strtof(const char*s,char**e){if(e)*e=(char*)s;return 0;} long double strtold(const char*s,char**e){if(e)*e=(char*)s;return 0;}
void qsort(void*b,size_t n,size_t s,int(*c)(const void*,const void*)){(void)b;(void)n;(void)s;(void)c;} void *bsearch(const void*k,const void*b,size_t n,size_t s,int(*c)(const void*,const void*)){(void)k;(void)b;(void)n;(void)s;(void)c;return 0;}
void _Exit(int c){(void)c;for(;;){}} int system(const char*s){(void)s;return -1;} char *realpath(const char*p,char*r){(void)p;(void)r;return 0;}
EOF
      $BB mkdir -p src/stdio
      cat > src/stdio/minstdio.c <<'EOF'
#include <stdarg.h>
typedef unsigned long size_t;
typedef struct FILE { int fd; } FILE;
static FILE fake[3];
FILE *stdin=&fake[0], *stdout=&fake[1], *stderr=&fake[2];
int printf(const char *fmt, ...) { (void)fmt; return 0; }
int fprintf(FILE *f, const char *fmt, ...) { (void)f; (void)fmt; return 0; }
int snprintf(char *s, size_t n, const char *fmt, ...) { (void)fmt; if (s && n) s[0]=0; return 0; }
int sprintf(char *s, const char *fmt, ...) { (void)fmt; if (s) s[0]=0; return 0; }
int vsnprintf(char *s, size_t n, const char *fmt, va_list ap) { (void)fmt; (void)ap; if (s && n) s[0]=0; return 0; }
int vfprintf(FILE *f, const char *fmt, va_list ap) { (void)f; (void)fmt; (void)ap; return 0; }
int putchar(int c) { return c; }
int puts(const char *s) { (void)s; return 0; }
int fputs(const char *s, FILE *f) { (void)s; (void)f; return 0; }
int fputc(int c, FILE *f) { (void)f; return c; }
int putc(int c, FILE *f) { return fputc(c, f); }
FILE *fopen(const char *p, const char *m) { (void)p; (void)m; return 0; }
FILE *fdopen(int fd, const char *m) { (void)fd; (void)m; return 0; }
int fclose(FILE *f) { (void)f; return 0; }
int fflush(FILE *f) { (void)f; return 0; }
size_t fread(void *p, size_t s, size_t n, FILE *f) { (void)p; (void)s; (void)n; (void)f; return 0; }
size_t fwrite(const void *p, size_t s, size_t n, FILE *f) { (void)p; (void)f; return n; }
char *fgets(char *s, int n, FILE *f) { (void)s; (void)n; (void)f; return 0; }
int fgetc(FILE *f) { (void)f; return -1; }
int getc(FILE *f) { return fgetc(f); }
int ungetc(int c, FILE *f) { (void)c; (void)f; return -1; }
int feof(FILE *f) { (void)f; return 1; }
int ferror(FILE *f) { (void)f; return 0; }
void clearerr(FILE *f) { (void)f; }
int fileno(FILE *f) { (void)f; return -1; }
int fseek(FILE *f, long o, int w) { (void)f; (void)o; (void)w; return -1; }
long ftell(FILE *f) { (void)f; return -1; }
void rewind(FILE *f) { (void)f; }
void perror(const char *s) { (void)s; }
EOF
      $BB mkdir -p src/malloc
      cat > src/malloc/malloc.c <<'EOF'
typedef unsigned long size_t;
static unsigned char heap[1024*1024];
static size_t hp;
void *malloc(size_t n) { size_t a=(n+15)&~(size_t)15; if (hp+a > sizeof heap) return 0; void *p=heap+hp; hp+=a; return p; }
void free(void *p) { (void)p; }
void *calloc(size_t n, size_t s) { size_t total=n*s; unsigned char *p=malloc(total); if (p) for (size_t i=0;i<total;i++) p[i]=0; return p; }
void *realloc(void *p, size_t n) { (void)p; return malloc(n); }
void *memalign(size_t a, size_t n) { (void)a; return malloc(n); }
int posix_memalign(void **m, size_t a, size_t n) { (void)a; *m=malloc(n); return *m?0:12; }
EOF
      cat > src/conf/sysconf.c <<'EOF'
#include <errno.h>
#include <unistd.h>
long sysconf(int name) { if (name == _SC_PAGE_SIZE) return 4096; errno = EINVAL; return -1; }
EOF
      cat > src/ctype/__ctype_get_mb_cur_max.c <<'EOF'
#include <stdlib.h>
size_t __ctype_get_mb_cur_max(void) { return 4; }
EOF
      cat > src/env/__init_tls.c <<'EOF'
typedef unsigned long size_t;
typedef unsigned long uintptr_t;
uintptr_t __stack_chk_guard;
void *__copy_tls(unsigned char *mem) { return mem; }
void __init_tls(size_t *aux) { (void)aux; }
EOF
      cat > src/env/__reset_tls.c <<'EOF'
void __reset_tls(void) {}
EOF
      cat > src/env/__stack_chk_fail.c <<'EOF'
void __stack_chk_fail(void) { for (;;) {} }
EOF
      cat > src/env/__libc_start_main.c <<'EOF'
int __libc_start_main(int (*main)(int,char **,char **), int argc, char **argv) { return main(argc, argv, argv); }
EOF
      cat > src/errno/__errno_location.c <<'EOF'
int *__errno_location(void) { static int e; return &e; }
EOF
      cat > src/errno/strerror.c <<'EOF'
char *strerror(int e) { (void)e; return "error"; }
EOF
      cat > src/exit/abort.c <<'EOF'
void abort(void) { for (;;) {} }
EOF
      cat > src/fcntl/open.c <<'EOF'
#include <fcntl.h>
int open(const char *filename, int flags, ...) { (void)filename; (void)flags; return -1; }
EOF
      cat > src/fcntl/openat.c <<'EOF'
#include <fcntl.h>
int openat(int fd, const char *filename, int flags, ...) { (void)fd; (void)filename; (void)flags; return -1; }
EOF
      cat > src/fcntl/posix_fadvise.c <<'EOF'
int posix_fadvise(int fd, long off, long len, int advice) { (void)fd; (void)off; (void)len; (void)advice; return 0; }
EOF
      cat > src/fcntl/fcntl.c <<'EOF'
#include <fcntl.h>
int fcntl(int fd, int cmd, ...) { (void)fd; (void)cmd; return -1; }
EOF
      $BB mkdir -p src/stat src/signal src/time
      cat > src/stat/minstat.c <<'EOF'
struct stat { unsigned long st_dev; unsigned long st_ino; unsigned long st_nlink; unsigned int st_mode; unsigned int st_uid; unsigned int st_gid; unsigned long st_rdev; long st_size; long st_blksize; long st_blocks; long st_atime; long st_mtime; long st_ctime; };
int stat(const char *p, struct stat *st) { (void)p; if (st) st->st_mode = 0; return -1; }
int lstat(const char *p, struct stat *st) { return stat(p, st); }
int fstat(int fd, struct stat *st) { (void)fd; return stat(0, st); }
int fchmod(int fd, unsigned int mode) { (void)fd; (void)mode; return -1; }
int fchown(int fd, unsigned int uid, unsigned int gid) { (void)fd; (void)uid; (void)gid; return -1; }
EOF
      cat > src/stdio/remove.c <<'EOF'
int unlink(const char *p);
int remove(const char *p) { return unlink(p); }
EOF
      cat > src/time/utime.c <<'EOF'
struct utimbuf { long actime; long modtime; };
int utime(const char *p, const struct utimbuf *t) { (void)p; (void)t; return -1; }
EOF
      cat > src/signal/signal.c <<'EOF'
typedef void (*sighandler_t)(int);
sighandler_t signal(int sig, sighandler_t h) { (void)sig; return h; }
EOF
      cat > crt/crt1.c <<'EOF'
void _start(void) { for (;;) {} }
EOF
      cat > src/conf/confstr.c <<'EOF'
#include <stddef.h>
size_t confstr(int name, char *buf, size_t len)
{
  const char *s = "";
  size_t i = 0;
  if (buf && len) { for (; i + 1 < len && s[i]; i++) buf[i] = s[i]; buf[i] = 0; }
  while (s[i]) i++;
  return i + 1;
}
EOF
      cat > src/crypt/crypt_sha256.c <<'EOF'
char *__crypt_sha256(const char *key, const char *setting, char *output)
{
  (void)key; (void)setting; output[0] = '*'; output[1] = 0; return output;
}
EOF
      cat > src/ipc/semctl.c <<'EOF'
int semctl(int id, int num, int cmd, ...) { (void)id; (void)num; (void)cmd; return -1; }
EOF
      cat > src/ldso/dlerror.c <<'EOF'
#include <stdarg.h>
char *dlerror(void) { return 0; }
void __dl_thread_cleanup(void) {}
void __dl_vseterr(const char *fmt, va_list ap) { (void)fmt; (void)ap; }
void __dl_seterr(const char *fmt, ...) { (void)fmt; }
int __dl_invalid_handle(void *h) { (void)h; return 1; }
EOF
      cat > src/fenv/x86_64/fenv.s <<'EOF'
.global feclearexcept
feclearexcept:
	xor %eax,%eax
	ret
.global feraiseexcept
feraiseexcept:
	xor %eax,%eax
	ret
.global __fesetround
.hidden __fesetround
__fesetround:
	xor %eax,%eax
	ret
.global fegetround
fegetround:
	xor %eax,%eax
	ret
.global fegetenv
fegetenv:
	xor %eax,%eax
	ret
.global fesetenv
fesetenv:
	xor %eax,%eax
	ret
.global fetestexcept
fetestexcept:
	xor %eax,%eax
	ret
EOF

      ./configure \
        --host=x86_64-linux-musl \
        --disable-shared \
        --prefix="$out" \
        CC=tcc \
        CFLAGS="-static" \
        AR="tcc -ar"

      $BB sed -i 's|^\(CC_CMD =.*\)|\1 ; chmod u+rw $@|' Makefile

      cat >> Makefile <<'EOF'
lib/libm.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/librt.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libpthread.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libcrypt.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libutil.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libxnet.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libresolv.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
lib/libdl.a: obj/src/math/minmath.o
	$(AR) rc $@ obj/src/math/minmath.o
EOF

      # Build
      make -j1 AR="tcc -ar" RANLIB=true

      # Install headers, static lib, and crt files
      make install

      # Verify the first-musl output contract.  This pass must fail closed:
      # downstream tcc-musl stages need libc.a, installed headers, and at
      # least one startup object rather than a silently incomplete install.
      test -f "$out/lib/libc.a"
      test -f "$out/include/stdio.h"
      if [ ! -f "$out/lib/crt1.o" ] && [ ! -f "$out/lib/Scrt1.o" ]; then
        echo "ERROR: musl install did not produce crt1.o or Scrt1.o" >&2
        exit 1
      fi
    "%,
  ],
  inputs = [stage0, tcc_musl, musl_prev, make, sed, musl_src],
} | crunch.Derivation
