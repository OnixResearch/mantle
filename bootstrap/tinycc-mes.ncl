# Build tinycc 0.9.26 using the mes C compiler (mescc).
#
# This is the first real C compiler in the chain. mescc (running on
# mes-m2) compiles tcc 0.9.26 from C source, then that tcc-mes binary
# compiles tcc-boot0 as the first self-compile boundary. The final shipped
# tcc is still emitted by Mes/mescc with final output runtime paths because
# tcc-boot0 is not yet reliable enough for the old boot0 → boot1 → boot2
# chain.
#
# Output: tcc-0.9.26 binary + mes libc rebuilt with tcc-mes.
#
# Adapted from live-bootstrap steps/tcc-0.9.26/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-0.27.1-src",
} in

let tcc_src = crunch.fetchTarball {
  url = "https://lilypond.org/janneke/tcc/tcc-0.9.26-1147-gee75a10c.tar.gz",
  hash = "sha256-Bkmpev9x8vKh2P9X8Z4VSpcy4BJtzc0Tra3WPImnuhc=",
  name = "tcc-0.9.26-src",
} in

{
  name = "tinycc-0.9.26",
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

      apply_tcctools_ar_patch() {
        tcc_tools=$1
        $BB sed -i '/if ((fh = fopen(argv\[i_lib\], "wb")) == NULL)/,/    }/d' "$tcc_tools"
        $BB sed -i '/^[[:space:]]*\/\/ write header/i\
    if ((fh = fopen(argv[i_lib], "wb")) == NULL)\
    {\
        fprintf(stderr, "tcc: ar: cannot open file %s \\n", argv[i_lib]);\
        goto the_end;\
    }\
' "$tcc_tools"
      }

      normalize_mescc_source_edges() {
        # mescc's type resolver is patched in bootstrap/mes.ncl so it can
        # handle TCC's self-referential BufferedFile typedef without mutating
        # the source-level struct shape. Keeping the original type layout here
        # avoids producing a runnable-but-defective boot compiler.

        # Mes 0.27.1 treats array parameters as scalar locals in this bootstrap
        # path, so `filename[0]` is code-generated as a non-pointer array-ref.
        # Spell the single affected TCC parameter in the decayed pointer form.
        $BB sed -i 's/const char filename\[\]/const char *filename/' tccelf.c

        # Mes 0.27.1's local initializer path only emits scalar stores. The
        # x86_64 call shuffler copies the 42-byte SValue struct through a local
        # initializer and direct struct assignments; route those copies through
        # memcpy so mescc emits byte-safe calls instead of a bogus scalar store.
        $BB sed -i 's/SValue tmp = vtop\[0\];/SValue tmp;\
            memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/tmp = vtop\[0\];/memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[0\] = vtop\[-i\];/memcpy(\&vtop[0], \&vtop[-i], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[-i\] = tmp;/memcpy(\&vtop[-i], \&tmp, sizeof(SValue));/' x86_64-gen.c

        # tcc-mes miscompiles variable doublings through the power-of-two
        # multiply shortcut as `shl $0`. Spell bootstrap allocator growth as
        # addition so boot0 does not loop forever in the first CString resize.
        $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
        $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
        $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
        $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
        $BB sed -i 's/loc = 0;/loc = 0;\
    ind = cur_text_section->data_offset;/' x86_64-gen.c
        $BB sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c

        # The Mes/TCC varargs path corrupts snprintf/sprintf string expansion
        # in the first boot compiler. Spell the bootstrap path constructors and
        # preprocessor base-file string as direct string appends.
        $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
        $BB sed -i '/sprintf(buf,/c\
    buf[0] = '\''"'\''; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
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

        # The first boot compiler preloads crt/libc objects before compiling
        # ONE_SOURCE tcc.c. Their untyped object symbols can conflict with Mes
        # header prototypes during declaration merging; keep bootstrap moving
        # by accepting those redefinitions instead of entering the broken
        # varargs error-reporting path.
        $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c

        # tcc-mes emits x86_64 PLT entries even for the static boot compiler.
        # Relocate those PLT slots after final section layout; otherwise the
        # generated boot0 binary jumps through unrelocated offsets into .plt.
        $BB sed -i '/if (file_type == TCC_OUTPUT_EXE && s1->static_link)/,/        fill_got(s1);/c\
    if (file_type == TCC_OUTPUT_EXE && s1->static_link) {\
        if (s1->plt)\
            relocate_plt(s1);\
        fill_got(s1);\
    }' tccelf.c
      }

      append_abort_object() {
        archive=$1
        if $BB grep -q '^:abort$' "$archive"; then
          return
        fi
        $BB cat >> "$archive" << 'EOF'
<
:abort
55
4889e5
4881ec A8200000
48c7c7 06000000
57
e8 %raise
4883c4 08
48c7c7 00000000
4829f8
0f9cc0
480fb6c0
4885c0
0f84 %_abort_1_break
48c7c0 00000000
488945 F8
48c7c0 02000000
488b7d F8
8807
4885c0
e9 %_abort_1_break
:_abort_1_break
4889ec
5d
c3
EOF
      }

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

      rebuild_mes_runtime() {
        compiler=$1
        mode=${2:-full}
        echo "tinycc-mes: rebuild-mes-runtime compiler=$compiler mode=$mode" >&2
        cd "$MES_BUILD"
        if [ "$mode" = full-no-crt ]; then
          echo "tinycc-mes: reuse crt objects" >&2
        else
          echo "tinycc-mes: compile crt1.o" >&2
          "$compiler" -c -D HAVE_CONFIG_H=1 -I include -I include/linux/${MES_ARCH} \
            -o "$LIBDIR/crt1.o" lib/linux/${MES_ARCH}-mes-gcc/crt1.c
          : > "$LIBDIR/crtn.o"
          : > "$LIBDIR/crti.o"
        fi
        echo "tinycc-mes: compile libtcc1.o" >&2
        "$compiler" -c -D HAVE_CONFIG_H=1 -D HAVE_LONG_LONG=1 -D HAVE_FLOAT=1 \
          -I include -I include/linux/${MES_ARCH} -o libtcc1.o lib/libtcc1.c
        $BB rm -f "$LIBDIR/tcc/libtcc1.a"
        "$compiler" -ar cr "$LIBDIR/tcc/libtcc1.a" libtcc1.o
        if [ "$mode" = full ] || [ "$mode" = full-no-crt ]; then
          echo "tinycc-mes: compile unified-libc.o" >&2
          "$compiler" -c -D HAVE_CONFIG_H=1 -I include -I include/linux/${MES_ARCH} \
            -o unified-libc.o unified-libc.c
          $BB rm -f "$LIBDIR/libc.a"
          "$compiler" -ar cr "$LIBDIR/libc.a" unified-libc.o
          echo "tinycc-mes: compile getopt.o" >&2
          "$compiler" -c -D HAVE_CONFIG_H=1 -I include -I include/linux/${MES_ARCH} \
            -o getopt.o lib/posix/getopt.c
          $BB rm -f "$LIBDIR/libgetopt.a"
          "$compiler" -ar cr "$LIBDIR/libgetopt.a" getopt.o
        fi
        cd "$TCC_BUILD"
      }

      compile_tcc() {
        compiler=$1
        output=$2
        runtime_libdir=${3:-$LIBDIR}
        runtime_incdir=${4:-$INCDIR}
        echo "tinycc-mes: compile $output with $compiler" >&2
        "$compiler" \
          -static \
          -o "$output" \
          -D BOOTSTRAP=1 \
          -D HAVE_FLOAT=1 \
          -D HAVE_BITFIELD=1 \
          -D HAVE_LONG_LONG=${HAVE_LONG_LONG} \
          -D HAVE_SETJMP=1 \
          -I . \
          -I "$INCDIR" \
          -D TCC_TARGET_${TCC_TARGET_ARCH}=1 \
          -D CONFIG_TCCDIR=\"$runtime_libdir/tcc\" \
          -D CONFIG_TCC_CRTPREFIX=\"$runtime_libdir\" \
          -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
          -D CONFIG_TCC_LIBPATHS=\"$runtime_libdir:$runtime_libdir/tcc\" \
          -D CONFIG_TCC_SYSINCLUDEPATHS=\"$runtime_incdir\" \
          -D TCC_LIBGCC=\"$runtime_libdir/libc.a\" \
          -D TCC_LIBTCC1=\"libtcc1.a\" \
          -D CONFIG_TCCBOOT=1 \
          -D CONFIG_TCC_STATIC=1 \
          -D CONFIG_USE_LIBGCC=1 \
          -D TCC_VERSION=\"0.9.26\" \
          -D ONE_SOURCE=1 \
          -L . \
          -L "$LIBDIR" \
          tcc.c
        $BB chmod 755 "$output"
      }

      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      MES_SRC=$(find_input mes-0.27.1-src)
      TCC_SRC=$(find_input tcc-0.9.26-src)

      MES_ARCH=x86_64
      TCC_TARGET_ARCH=X86_64
      HAVE_LONG_LONG=1
      MES_TCC_BASE_ADDRESS=0x08048000

      export MES_STACK=15000000
      export MES_ARENA=30000000
      export MES_MAX_ARENA=30000000
      MES="$MES_OUT/bin/mes-m2"
      MESCC="$MES_OUT/bin/mescc.scm"

      WORK=/build/tcc-build
      PREFIX="$WORK/prefix"
      BINDIR="$PREFIX/bin"
      LIBDIR="$PREFIX/lib/mes"
      INCDIR="$PREFIX/include/mes"
      MES_LINK_ROOT="$WORK/mes-link"
      TCC_BUILD="$WORK/tcc-0.9.26"
      MES_BUILD="$WORK/mes-0.27.1"

      export PATH="$BINDIR:$STAGE0/bin:$MES_OUT/bin:$PATH"
      export MES_PREFIX="$MES_OUT"
      export GUILE_LOAD_PATH="$MES_OUT/mes/module"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$BINDIR" "$LIBDIR/tcc" "$PREFIX/include" "$MES_LINK_ROOT/linux"

      $BB cp -r "$TCC_SRC" "$TCC_BUILD"
      $BB cp -r "$MES_SRC" "$MES_BUILD"
      $BB chmod -R u+w "$TCC_BUILD" "$MES_BUILD"

      $BB cp -r "$MES_OUT/include/mes-include" "$INCDIR"
      $BB cp "$MES_OUT/lib/${MES_ARCH}-mes"/* "$LIBDIR/"
      $BB cp -r "$MES_OUT/lib/${MES_ARCH}-mes" "$MES_LINK_ROOT/"
      $BB cp -r "$MES_OUT/lib/linux/${MES_ARCH}-mes" "$MES_LINK_ROOT/linux/"
      $BB chmod -R u+w "$MES_LINK_ROOT"
      append_abort_object "$MES_LINK_ROOT/${MES_ARCH}-mes/libc+tcc.a"

      $BB cat > "$MES_BUILD/include/mes/config.h" << 'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF
      $BB sed -i '/^int oputs (char const \*s);$/d' "$MES_BUILD/include/mes/lib.h"
      $BB mkdir -p "$MES_BUILD/include/arch"
      $BB cp "$MES_BUILD/include/linux/${MES_ARCH}/kernel-stat.h" "$MES_BUILD/include/arch/kernel-stat.h"
      $BB cp "$MES_BUILD/include/linux/${MES_ARCH}/signal.h" "$MES_BUILD/include/arch/signal.h"
      $BB cp "$MES_BUILD/include/linux/${MES_ARCH}/syscall.h" "$MES_BUILD/include/arch/syscall.h"
      create_unified_libc

      cd "$TCC_BUILD"
      : > config.h
      apply_tcctools_ar_patch tcctools.c
      normalize_mescc_source_edges

      ################################################################
      # Step 1: Compile tcc with mescc (Scheme-based C compiler)     #
      ################################################################
      echo "tinycc-mes: mescc emit tcc.s" >&2
      "$MES" --no-auto-compile -e main "$MESCC" -- \
        -S \
        -o tcc.s \
        -I "$INCDIR" \
        -D BOOTSTRAP=1 \
        -D HAVE_LONG_LONG=${HAVE_LONG_LONG} \
        -I . \
        -D TCC_TARGET_${TCC_TARGET_ARCH}=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"$LIBDIR/tcc\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$LIBDIR\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_SYSINCLUDEPATHS=\"$INCDIR\" \
        -D TCC_LIBGCC=\"$LIBDIR/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c

      echo "tinycc-mes: mescc link tcc-mes" >&2
      libdir="$MES_LINK_ROOT" "$MES" --no-auto-compile -e main "$MESCC" -- \
        --base-address "$MES_TCC_BASE_ADDRESS" \
        -o tcc-mes \
        -L "$MES_LINK_ROOT" \
        tcc.s \
        -l c+tcc
      $BB chmod 755 tcc-mes
      $BB cp tcc-mes "$BINDIR/tcc-mes"
      tcc-mes -version

      ################################################################
      # Step 2: Rebuild Mes libc, then bootstrap tcc                 #
      ################################################################
      rebuild_mes_runtime "$BINDIR/tcc-mes"

      compile_tcc "$BINDIR/tcc-mes" tcc-boot0
      $BB cp tcc-boot0 "$BINDIR/tcc-boot0"
      "$BINDIR/tcc-boot0" -version

      ################################################################
      # Install                                                       #
      ################################################################
      $BB mkdir -p "$out/bin" "$out/lib/mes/tcc" "$out/include"
      $BB cp -r "$INCDIR" "$out/include/mes"
      $BB cp -r "$INCDIR" "$out/include/mes-include"
      $BB cp "$LIBDIR/crt1.o" "$out/lib/mes/crt1.o"
      $BB cp "$LIBDIR/crtn.o" "$out/lib/mes/crtn.o"
      $BB cp "$LIBDIR/crti.o" "$out/lib/mes/crti.o"
      $BB cp "$LIBDIR/libc.a" "$out/lib/mes/libc.a"
      $BB cp "$LIBDIR/libgetopt.a" "$out/lib/mes/libgetopt.a"
      $BB cp "$LIBDIR/tcc/libtcc1.a" "$out/lib/mes/tcc/libtcc1.a"

      echo "tinycc-mes: mescc emit final tcc.s" >&2
      "$MES" --no-auto-compile -e main "$MESCC" -- \
        -S \
        -o tcc-final.s \
        -I "$INCDIR" \
        -D BOOTSTRAP=1 \
        -D HAVE_LONG_LONG=${HAVE_LONG_LONG} \
        -I . \
        -D TCC_TARGET_${TCC_TARGET_ARCH}=1 \
        -D inline= \
        -D CONFIG_TCCDIR=\"$out/lib/mes/tcc\" \
        -D CONFIG_SYSROOT=\"/\" \
        -D CONFIG_TCC_CRTPREFIX=\"$out/lib/mes\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_LIBPATHS=\"$out/lib/mes:$out/lib/mes/tcc\" \
        -D CONFIG_TCC_SYSINCLUDEPATHS=\"$out/include/mes\" \
        -D TCC_LIBGCC=\"$out/lib/mes/libc.a\" \
        -D CONFIG_TCC_LIBTCC1_MES=0 \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26\" \
        -D ONE_SOURCE=1 \
        tcc.c
      echo "tinycc-mes: mescc link final tcc" >&2
      libdir="$MES_LINK_ROOT" "$MES" --no-auto-compile -e main "$MESCC" -- \
        --base-address "$MES_TCC_BASE_ADDRESS" \
        -o "$out/bin/tcc-0.9.26" \
        -L "$MES_LINK_ROOT" \
        tcc-final.s \
        -l c+tcc
      $BB cp "$out/bin/tcc-0.9.26" "$out/bin/tcc"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.26"
      "$out/bin/tcc" -version

      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.26"
      test -s "$out/lib/mes/libc.a"
      test -s "$out/lib/mes/tcc/libtcc1.a"
    "%,
  ],
  inputs = [stage0, mes, mes_src, tcc_src],
} | crunch.Derivation
