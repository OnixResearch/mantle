# Spike proof: create an i386 Mes runtime/header layout for tcc26 -> tcc27.
#
# This is intentionally a sibling diagnostic derivation. It imports the proven
# no-libc i386 TinyCC 0.9.26 handoff and records the first concrete blocker
# before any production make-tcc routing changes.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26_i386 = import "spike-i386-tinycc26-cross-smoke.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-0.27.1-src",
} in

let tcc27_src = crunch.fetchTarball {
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

let make_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/make/make-3.82.tar.bz2",
  hash = "sha256-GwvLvcEtBMdaC/3YqmUkE9aoNTmrj9yQ39Vyd+Fw+M4=",
  name = "make-3.82-src",
} in

{
  name = "spike-i386-mes-runtime-layout",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
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

      STAGE0=$(find_input stage0-posix)
      TCC26=$(find_input spike-i386-tinycc26-cross-smoke)
      MES_SRC=$(find_input mes-0.27.1-src)
      TCC27_SRC=$(find_input tcc-0.9.27-src)
      MAKE_SRC=$(find_input make-3.82-src)
      export PATH="$TCC26/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/spike-i386-mes-runtime-layout
      LOGS=$WORK/logs
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$LOGS"
      cd "$WORK"

      run_step() {
        name="$1"
        shift
        echo "i386-tcc27-make: STEP $name: $*" >&2
        set +e
        "$@" >"$LOGS/$name.stdout" 2>"$LOGS/$name.stderr"
        rc=$?
        set -e
        echo "$rc" >"$LOGS/$name.rc"
        echo "i386-tcc27-make: RESULT $name rc=$rc" >&2
        return "$rc"
      }

      note_blocker() {
        step="$1"
        rc_file="$LOGS/$step.rc"
        rc=missing
        if [ -f "$rc_file" ]; then rc=$($BB cat "$rc_file"); fi
        $BB mkdir -p "$out/share/spike-i386-mes-runtime-layout"
        $BB cp -r "$LOGS" "$out/share/spike-i386-mes-runtime-layout/logs"
        $BB mkdir -p "$out/share/spike-i386-mes-runtime-layout/artifacts"
        [ -f "$WORK/tcc-0.9.27/tcc27.o" ] && $BB cp "$WORK/tcc-0.9.27/tcc27.o" "$out/share/spike-i386-mes-runtime-layout/artifacts/tcc27.o" || true
        [ -f "$WORK/libtcc1.o" ] && $BB cp "$WORK/libtcc1.o" "$out/share/spike-i386-mes-runtime-layout/artifacts/libtcc1.o" || true
        [ -f "$RUNTIME/lib/mes/libc.a" ] && $BB cp "$RUNTIME/lib/mes/libc.a" "$out/share/spike-i386-mes-runtime-layout/artifacts/libc.a" || true
        [ -f "$RUNTIME/lib/mes/crt1.o" ] && $BB cp "$RUNTIME/lib/mes/crt1.o" "$out/share/spike-i386-mes-runtime-layout/artifacts/crt1.o" || true
        {
          echo "status=blocked"
          echo "blocked_step=$step"
          echo "blocked_rc=$rc"
          echo "note=The i386 Mes runtime/header layout proof found the first concrete blocker before tcc27 can use the layout."
        } > "$out/share/spike-i386-mes-runtime-layout/summary.txt"
      }

      echo 'i386-mes-layout: predecessor version' >&2
      run_step tcc26_version tcc26-i386 -version

      RUNTIME="$WORK/i386-mes-runtime"
      MES_BUILD="$WORK/mes-0.27.1"
      $BB cp -r "$MES_SRC" "$MES_BUILD"
      $BB chmod -R u+w "$MES_BUILD"
      $BB mkdir -p "$RUNTIME/include/mes" "$RUNTIME/lib/mes/tcc"
      $BB cp -r "$MES_BUILD/include" "$RUNTIME/include/mes"
      $BB mkdir -p "$RUNTIME/include/mes/include/arch"
      $BB cp "$MES_BUILD/include/linux/x86/kernel-stat.h" "$RUNTIME/include/mes/include/arch/kernel-stat.h"
      $BB cp "$MES_BUILD/include/linux/x86/signal.h" "$RUNTIME/include/mes/include/arch/signal.h"
      $BB cp "$MES_BUILD/include/linux/x86/syscall.h" "$RUNTIME/include/mes/include/arch/syscall.h"
      $BB cat > "$RUNTIME/include/mes/include/mes/config.h" << 'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF

      MES_ARCH=x86

      run_step runtime_header_smoke test -f "$RUNTIME/include/mes/include/stdio.h"
      if ! run_step runtime_crt1_object tcc26-i386 -c -D HAVE_CONFIG_H=1 -I "$RUNTIME/include/mes/include" -I "$RUNTIME/include/mes/include/linux/x86" -o "$RUNTIME/lib/mes/crt1.o" "$MES_BUILD/lib/linux/x86-mes-gcc/crt1.c"; then
        note_blocker runtime_crt1_object
        exit 0
      fi
      : > "$RUNTIME/lib/mes/crti.o"
      : > "$RUNTIME/lib/mes/crtn.o"
      if ! run_step runtime_libtcc1_object tcc26-i386 -c -D HAVE_CONFIG_H=1 -D TCC_TARGET_I386=1 -D HAVE_FLOAT=0 -D HAVE_FLOAT_STUB=0 -D HAVE_LONG_LONG=0 -I "$RUNTIME/include/mes/include" -I "$RUNTIME/include/mes/include/linux/x86" -o "$WORK/libtcc1.o" "$MES_BUILD/lib/libtcc1.c"; then
        echo 'i386-mes-layout: libtcc1 object is blocked; create placeholder archives to continue header-only tcc27 object probe' >&2
        $BB cp "$RUNTIME/lib/mes/crt1.o" "$WORK/libtcc1.o"
      fi
      if ! run_step runtime_libtcc1_archive tcc26-i386 -ar cr "$RUNTIME/lib/mes/tcc/libtcc1.a" "$WORK/libtcc1.o"; then
        note_blocker runtime_libtcc1_archive
        exit 0
      fi
      $BB rm -rf "$WORK/libc-objs"
      $BB mkdir -p "$WORK/libc-objs"
      LIBC_OBJS=""
      LIBC_FAILS=0
      for src in \
          ctype/isalnum.c ctype/isalpha.c ctype/isascii.c ctype/iscntrl.c ctype/isdigit.c ctype/isgraph.c \
          ctype/islower.c ctype/isnumber.c ctype/isprint.c ctype/ispunct.c ctype/isspace.c ctype/isupper.c \
          ctype/isxdigit.c ctype/tolower.c ctype/toupper.c dirent/closedir.c dirent/__getdirentries.c dirent/opendir.c \
          linux/readdir.c linux/access.c linux/brk.c linux/chdir.c linux/chmod.c linux/clock_gettime.c \
          linux/close.c linux/dup2.c linux/dup.c linux/execve.c linux/fcntl.c linux/fork.c \
          linux/fsync.c linux/fstat.c linux/_getcwd.c linux/getdents.c linux/getegid.c linux/geteuid.c \
          linux/getgid.c linux/getpid.c linux/getppid.c linux/getrusage.c linux/gettimeofday.c linux/getuid.c \
          linux/ioctl.c linux/ioctl3.c linux/kill.c linux/link.c linux/lseek.c linux/lstat.c \
          linux/malloc.c linux/mkdir.c linux/mknod.c linux/nanosleep.c linux/_open3.c linux/pipe.c \
          linux/_read.c linux/readlink.c linux/rename.c linux/rmdir.c linux/setgid.c linux/settimer.c \
          linux/setuid.c linux/signal.c linux/sigprogmask.c linux/symlink.c linux/stat.c linux/time.c \
          linux/unlink.c linux/waitpid.c linux/wait4.c linux/${MES_ARCH}-mes-gcc/_exit.c linux/${MES_ARCH}-mes-gcc/syscall.c linux/${MES_ARCH}-mes-gcc/_write.c \
          math/ceil.c math/fabs.c math/floor.c mes/abtod.c mes/abtol.c mes/__assert_fail.c \
          mes/assert_msg.c mes/__buffered_read.c mes/__init_io.c mes/cast.c mes/dtoab.c mes/eputc.c \
          mes/eputs.c mes/fdgetc.c mes/fdgets.c mes/fdputc.c mes/fdputs.c mes/fdungetc.c \
          mes/globals.c mes/itoa.c mes/ltoab.c mes/ltoa.c mes/__mes_debug.c mes/mes_open.c \
          mes/ntoab.c mes/oputc.c mes/oputs.c mes/search-path.c mes/ultoa.c mes/utoa.c \
          posix/alarm.c posix/buffered-read.c posix/execl.c posix/execlp.c posix/execv.c posix/execvp.c \
          posix/getcwd.c posix/getenv.c posix/isatty.c posix/mktemp.c posix/open.c posix/pathconf.c \
          posix/raise.c posix/sbrk.c posix/setenv.c posix/sleep.c posix/unsetenv.c posix/wait.c \
          posix/write.c stdio/clearerr.c stdio/fclose.c stdio/fdopen.c stdio/feof.c stdio/ferror.c \
          stdio/fflush.c stdio/fgetc.c stdio/fgets.c stdio/fileno.c stdio/fopen.c stdio/fprintf.c \
          stdio/fputc.c stdio/fputs.c stdio/fread.c stdio/freopen.c stdio/fscanf.c stdio/fseek.c \
          stdio/ftell.c stdio/fwrite.c stdio/getc.c stdio/getchar.c stdio/perror.c stdio/printf.c \
          stdio/putc.c stdio/putchar.c stdio/remove.c stdio/snprintf.c stdio/sprintf.c stdio/sscanf.c \
          stdio/ungetc.c stdio/vfprintf.c stdio/vfscanf.c stdio/vprintf.c stdio/vsnprintf.c stdio/vsprintf.c \
          stdio/vsscanf.c stdlib/abort.c stdlib/abs.c stdlib/alloca.c stdlib/atexit.c stdlib/atof.c \
          stdlib/atoi.c stdlib/atol.c stdlib/calloc.c stdlib/__exit.c stdlib/exit.c stdlib/free.c \
          stdlib/mbstowcs.c stdlib/puts.c stdlib/qsort.c stdlib/realloc.c stdlib/strtod.c stdlib/strtof.c \
          stdlib/strtol.c stdlib/strtold.c stdlib/strtoll.c stdlib/strtoul.c stdlib/strtoull.c string/bcmp.c \
          string/bcopy.c string/bzero.c string/index.c string/memchr.c string/memcmp.c string/memcpy.c \
          string/memmem.c string/memmove.c string/memset.c string/rindex.c string/strcat.c string/strchr.c \
          string/strcmp.c string/strcpy.c string/strcspn.c string/strdup.c string/strerror.c string/strlen.c \
          string/strlwr.c string/strncat.c string/strncmp.c string/strncpy.c string/strpbrk.c string/strrchr.c \
          string/strspn.c string/strstr.c string/strupr.c stub/atan2.c stub/bsearch.c stub/chown.c \
          stub/__cleanup.c stub/cos.c stub/ctime.c stub/exp.c stub/fpurge.c stub/freadahead.c \
          stub/frexp.c stub/getgrgid.c stub/getgrnam.c stub/getlogin.c stub/getpgid.c stub/getpgrp.c \
          stub/getpwnam.c stub/getpwuid.c stub/gmtime.c stub/ldexp.c stub/localtime.c stub/log.c \
          stub/mktime.c stub/modf.c stub/mprotect.c stub/pclose.c stub/popen.c stub/pow.c \
          stub/putenv.c stub/rand.c stub/realpath.c stub/rewind.c stub/setbuf.c stub/setgrent.c \
          stub/setlocale.c stub/setvbuf.c stub/sigaction.c stub/sigaddset.c stub/sigblock.c stub/sigdelset.c \
          stub/sigemptyset.c stub/sigsetmask.c stub/sin.c stub/sys_siglist.c stub/system.c stub/sqrt.c \
          stub/strftime.c stub/times.c stub/ttyname.c stub/umask.c stub/utime.c ${MES_ARCH}-mes-gcc/setjmp.c; do
        case "$src" in
          linux/x86-mes-gcc/_exit.c|linux/x86-mes-gcc/_write.c|linux/x86-mes-gcc/syscall.c|\${MES_ARCH}-mes-gcc/_exit.c|\${MES_ARCH}-mes-gcc/_write.c|\${MES_ARCH}-mes-gcc/syscall.c)
            continue
            ;;
        esac
        safe=$($BB echo "$src" | $BB sed 's|/|_|g; s|\.c$||')
        obj="$WORK/libc-objs/$safe.o"
        echo "i386-mes-layout: libc object $src" >&2
        set +e
        tcc26-i386 -c -D HAVE_CONFIG_H=1 -D TCC_TARGET_I386=1 -D HAVE_FLOAT=0 -D HAVE_FLOAT_STUB=0 -D HAVE_LONG_LONG=0 -I "$RUNTIME/include/mes/include" -I "$RUNTIME/include/mes/include/linux/x86" -o "$obj" "$MES_BUILD/lib/$src" >"$LOGS/runtime_libc_obj_$safe.stdout" 2>"$LOGS/runtime_libc_obj_$safe.stderr"
        rc=$?
        set -e
        echo "$rc" >"$LOGS/runtime_libc_obj_$safe.rc"
        if [ "$rc" = 0 ]; then
          LIBC_OBJS="$LIBC_OBJS $obj"
        else
          LIBC_FAILS=$((LIBC_FAILS + 1))
        fi
      done
      $BB cat > "$WORK/i386-syscalls.s" <<'EOF'
.globl _exit
_exit:
  movl $1, %eax
  movl 4(%esp), %ebx
  int $0x80
1: jmp 1b

.globl _write
_write:
  pushl %ebx
  movl $4, %eax
  movl 8(%esp), %ebx
  movl 12(%esp), %ecx
  movl 16(%esp), %edx
  int $0x80
  popl %ebx
  ret

.globl __sys_call
.globl _sys_call
__sys_call:
_sys_call:
  movl 4(%esp), %eax
  int $0x80
  ret

.globl __sys_call1
.globl _sys_call1
__sys_call1:
_sys_call1:
  pushl %ebx
  movl 8(%esp), %eax
  movl 12(%esp), %ebx
  int $0x80
  popl %ebx
  ret

.globl __sys_call2
.globl _sys_call2
__sys_call2:
_sys_call2:
  pushl %ebx
  movl 8(%esp), %eax
  movl 12(%esp), %ebx
  movl 16(%esp), %ecx
  int $0x80
  popl %ebx
  ret

.globl __sys_call3
.globl _sys_call3
__sys_call3:
_sys_call3:
  pushl %ebx
  movl 8(%esp), %eax
  movl 12(%esp), %ebx
  movl 16(%esp), %ecx
  movl 20(%esp), %edx
  int $0x80
  popl %ebx
  ret

.globl __sys_call4
.globl _sys_call4
__sys_call4:
_sys_call4:
  pushl %ebx
  pushl %esi
  movl 12(%esp), %eax
  movl 16(%esp), %ebx
  movl 20(%esp), %ecx
  movl 24(%esp), %edx
  movl 28(%esp), %esi
  int $0x80
  popl %esi
  popl %ebx
  ret

.globl _sys_call5
_sys_call5:
  pushl %ebx
  pushl %esi
  pushl %edi
  movl 16(%esp), %eax
  movl 20(%esp), %ebx
  movl 24(%esp), %ecx
  movl 28(%esp), %edx
  movl 32(%esp), %esi
  movl 36(%esp), %edi
  int $0x80
  popl %edi
  popl %esi
  popl %ebx
  ret

.globl _sys_call6
_sys_call6:
  pushl %ebp
  pushl %ebx
  pushl %esi
  pushl %edi
  movl 20(%esp), %eax
  movl 24(%esp), %ebx
  movl 28(%esp), %ecx
  movl 32(%esp), %edx
  movl 36(%esp), %esi
  movl 40(%esp), %edi
  movl 44(%esp), %ebp
  int $0x80
  popl %edi
  popl %esi
  popl %ebx
  popl %ebp
  ret

.globl __fixunsxfdi
__fixunsxfdi:
  xorl %eax, %eax
  xorl %edx, %edx
  ret

.globl __fixdfdi
__fixdfdi:
  xorl %eax, %eax
  xorl %edx, %edx
  ret

.globl __fixsfdi
__fixsfdi:
  xorl %eax, %eax
  xorl %edx, %edx
  ret

.globl __fixxfdi
__fixxfdi:
  xorl %eax, %eax
  xorl %edx, %edx
  ret

.globl __floatundixf
__floatundixf:
  fldz
  ret
EOF
      if ! run_step runtime_i386_syscalls_object tcc26-i386 -c -o "$WORK/libc-objs/i386-syscalls.o" "$WORK/i386-syscalls.s"; then
        note_blocker runtime_i386_syscalls_object
        exit 0
      fi
      $BB chmod 644 "$WORK/libc-objs/i386-syscalls.o"
      LIBC_OBJS="$LIBC_OBJS $WORK/libc-objs/i386-syscalls.o"
      echo "$LIBC_FAILS" > "$LOGS/runtime_libc_object_failures.rc"
      if [ "$LIBC_FAILS" != 0 ]; then
        note_blocker runtime_libc_object_failures
        exit 0
      fi
      $BB rm -f "$RUNTIME/lib/mes/libc.a"
      echo "i386-tcc27-make: STEP runtime_libc_archive: manual ar rc archive" >&2
      set +e
      {
        archive="$RUNTIME/lib/mes/libc.a"
        $BB rm -f "$archive"
        printf "!<arch>\n" > "$archive"
        i=0
        for f in $LIBC_OBJS; do
          i=$((i + 1))
          short="o$i.o"
          tmp="/tmp/$short"
          $BB cp "$f" "$tmp"
          size=$($BB wc -c < "$tmp" | $BB tr -d " ")
          printf '%-16s%-12s%-6s%-6s%-8s%-10s`\n' "$short/" 0 0 0 100644 "$size" >> "$archive"
          $BB cat "$tmp" >> "$archive"
          if [ $((size % 2)) != 0 ]; then printf "\n" >> "$archive"; fi
          $BB rm -f "$tmp"
        done
        test -s "$archive"
      } > "$LOGS/runtime_libc_archive.stdout" 2> "$LOGS/runtime_libc_archive.stderr"
      rc=$?
      set -e
      echo "$rc" > "$LOGS/runtime_libc_archive.rc"
      echo "i386-tcc27-make: RESULT runtime_libc_archive rc=$rc" >&2
      if [ "$rc" != 0 ]; then
        note_blocker runtime_libc_archive
        exit 0
      fi

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27
      : > config.h

      # Live-bootstrap source edits plus Crunch Mes/TCC normalization seams.
      $BB sed -i '/if ((fh = fopen(argv\[i_lib\], "wb")) == NULL)/,/    }/d' tcctools.c
      $BB sed -i '/^[[:space:]]*\/\/ write header/i\
    if ((fh = fopen(argv[i_lib], "wb")) == NULL)\
    {\
        fprintf(stderr, "tcc: ar: can'\''t open file %s \\n", argv[i_lib]);\
        goto the_end;\
    }\
' tcctools.c
      $BB sed -i '/if (ph->p_offset == 0) {/,/                    }/s/ph->p_paddr = ph->p_vaddr;/ph->p_paddr = ph->p_vaddr;\
                        if (s1->text_addr == 0x80100000)\
                            ph->p_paddr = ph->p_paddr - 0x80000000;/' tccelf.c
      # live-bootstrap tcc-0.9.27/pass1 check-reloc-null patch.
      $BB sed -i '/static void fill_local_got_entries(TCCState \*s1)/,/for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {/s/    ElfW_Rel \*rel;/    ElfW_Rel *rel;\
    if (!s1->got->reloc)\
        return;/' tccelf.c
      $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
      $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
      $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
      $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
      $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
      $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c

      $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
      $BB sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\
ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
{\
    char buf[1024];\
    int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o"))\
        return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) {\
        strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\
            return 0;\
    }\
    tcc_error_noabort("file '\''%s'\'' not found", filename);\
    return 0;\
}' libtcc.c
      $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
      $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
      $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
      $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rel"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
      $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c
      $BB awk '/tcc_error_noabort\("undefined symbol/ { print "                fputs(\"undefined symbol \", stderr); fputs(name, stderr); fputs(\"\\\\n\", stderr); s1->nb_errors++;"; next } { print }' tccelf.c > tccelf.c.new
      $BB mv tccelf.c.new tccelf.c
      $BB awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH: tcc26-i386 crashes compiling native x87 long-double initializer path */"; done = 1; next } { print }' tccgen.c > tccgen.c.new
      $BB mv tccgen.c.new tccgen.c

      # Focused predecessor diagnostics. These are non-gating: they preserve a
      # narrow map of where tcc26-i386 still falls over before the gating pass1
      # compile below records the first hard blocker.
      TCC27_FLAGS="-I . -I $RUNTIME/include/mes/include -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\\\"$RUNTIME/lib/mes/tcc\\\" -D CONFIG_TCC_CRTPREFIX=\\\"$RUNTIME/lib/mes\\\" -D CONFIG_TCC_ELFINTERP=\\\"/mes/loader\\\" -D CONFIG_TCC_LIBPATHS=\\\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\\\" -D CONFIG_TCC_SYSINCLUDEPATHS=\\\"$RUNTIME/include/mes/include\\\" -D TCC_LIBGCC=\\\"$RUNTIME/lib/mes/libc.a\\\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\\\"0.9.27-i386-spike\\\""
      run_tcc27_diag() {
        diag_name="$1"
        shift
        run_step "$diag_name" /bin/sh -c "tcc26-i386 $TCC27_FLAGS $*"
      }
      run_tcc27_diag tcc27_preprocess_no_lines "-E -P -D ONE_SOURCE=1 tcc.c" || true
      run_tcc27_diag tcc27_preprocess_line_markers "-E -D ONE_SOURCE=1 tcc.c" || true
      run_tcc27_diag tcc27_compile_tccpp_unit "-c -o tccpp.o tccpp.c" || true
      run_tcc27_diag tcc27_compile_tccelf_unit "-c -o tccelf.o tccelf.c" || true
      run_tcc27_diag tcc27_compile_i386_gen_unit "-c -o i386-gen.o i386-gen.c" || true
      run_tcc27_diag tcc27_compile_libtcc_unit "-c -o libtcc.o -D ONE_SOURCE=0 libtcc.c" || true
      run_tcc27_diag tcc27_compile_tccgen_unit "-c -o tccgen.o tccgen.c" || true

      cd "$WORK"
      $BB rm -rf tcc-0.9.27-decl-stub
      $BB cp -r tcc-0.9.27 tcc-0.9.27-decl-stub
      cd tcc-0.9.27-decl-stub
      $BB awk 'NR == 6638 { print "    (void)type; (void)sec; (void)c; (void)first; (void)size_only;"; print "#if 0" } NR == 6809 { print "#endif" } { print }' tccgen.c > tccgen.c.new
      $BB mv tccgen.c.new tccgen.c
      run_tcc27_diag tcc27_decl_initializer_stub_tccgen "-c -o tccgen.o tccgen.c" || true
      run_tcc27_diag tcc27_decl_initializer_stub_full "-c -o tcc27.o -D ONE_SOURCE=1 tcc.c" || true
      cd "$WORK/tcc-0.9.27"

      # First prove the predecessor can at least parse/compile 0.9.27 to an i386 object.
      if ! run_step tcc27_compile_object tcc26-i386 -c -I . -I "$RUNTIME/include/mes/include" -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c -o tcc27.o; then
        cd "$WORK"
        note_blocker tcc27_compile_object
        exit 0
      fi
      if ! run_step tcc27_object_exists test -f tcc27.o; then
        cd "$WORK"
        note_blocker tcc27_object_exists
        exit 0
      fi

      # Link/search-path diagnostics for the post-compile blocker. Keep these
      # non-gating so the evidence bundle captures which path still reaches the
      # Mes/TCC varargs-broken startup/library diagnostics before the canonical
      # source link below records the hard blocker.
      run_step tcc27_link_object_nostdlib_crt1 tcc26-i386 -nostdlib -static -o tcc27-i386.nostdlib-crt1 "$RUNTIME/lib/mes/crt1.o" tcc27.o "$RUNTIME/lib/mes/tcc/libtcc1.a" "$RUNTIME/lib/mes/libc.a" || true
      run_step tcc27_link_object_nostdlib_objects tcc26-i386 -nostdlib -static -o tcc27-i386.nostdlib-objects "$RUNTIME/lib/mes/crt1.o" tcc27.o "$WORK/libtcc1.o" $LIBC_OBJS || true
      run_step tcc27_link_object_nostdlib_objects_defs_first tcc26-i386 -nostdlib -static -o tcc27-i386.nostdlib-objects-defs-first "$RUNTIME/lib/mes/crt1.o" $LIBC_OBJS "$WORK/libtcc1.o" tcc27.o $LIBC_OBJS "$WORK/libtcc1.o" || true
      run_step tcc27_link_object_B_L tcc26-i386 -static -B "$RUNTIME/lib/mes" -L "$RUNTIME/lib/mes" -L "$RUNTIME/lib/mes/tcc" -o tcc27-i386.object-BL tcc27.o || true
      run_step tcc27_link_source_B_L tcc26-i386 -static -B "$RUNTIME/lib/mes" -L "$RUNTIME/lib/mes" -L "$RUNTIME/lib/mes/tcc" -o tcc27-i386.source-BL -I . -I "$RUNTIME/include/mes/include" -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c || true
      run_step tcc27_link_canonical_diagnostic tcc26-i386 -static -o tcc27-i386.canonical -I . -I "$RUNTIME/include/mes/include" -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c || true

      if ! run_step tcc27_link tcc26-i386 -nostdlib -static -o tcc27-i386 "$RUNTIME/lib/mes/crt1.o" tcc27.o "$WORK/libtcc1.o" $LIBC_OBJS; then
        cd "$WORK"
        note_blocker tcc27_link
        exit 0
      fi
      $BB chmod 755 tcc27-i386
      if ! run_step tcc27_version ./tcc27-i386 -version; then
        cd "$WORK"
        note_blocker tcc27_version
        exit 0
      fi

      cd "$WORK"
      $BB cp -r "$MAKE_SRC" make-3.82
      $BB chmod -R u+w make-3.82
      cd make-3.82
      : > config.h
      if ! run_step make_compile_getopt ../tcc-0.9.27/tcc27-i386 -c getopt.c; then
        cd "$WORK"
        note_blocker make_compile_getopt
        exit 0
      fi

      cd "$WORK"
      $BB mkdir -p "$out/bin" "$out/share/spike-i386-mes-runtime-layout"
      $BB cp tcc-0.9.27/tcc27-i386 "$out/bin/tcc27-i386" 2>/dev/null || true
      $BB cp -r "$LOGS" "$out/share/spike-i386-mes-runtime-layout/logs"
      printf 'status=partial\nblocked_step=make_full_smoke_not_yet_implemented\n' > "$out/share/spike-i386-mes-runtime-layout/summary.txt"
    "%,
  ],
  inputs = [stage0, tcc26_i386, mes_src, tcc27_src, make_src],
} | crunch.Derivation
