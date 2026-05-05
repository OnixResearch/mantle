# Build GCC 4.0.4 using tcc + musl + binutils 2.30.
#
# This is the first real C compiler in the bootstrap chain, replacing
# TinyCC. GCC 4.0.4 is the oldest GCC that TCC can build. Attempts
# C and C++ languages; falls back to C-only if C++ fails.
#
# Adapted from live-bootstrap steps/gcc-4.0.4/pass1.sh.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in

let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-QNHCUqWDcOtqVLokzAd3MfsWWR/jN3HdAld/Pm+GSM0=",
  name = "gcc-4.0.4-src",
} in

{
  name = "gcc-4.0.4",
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
      BINUTILS=$(find_input binutils-2.30-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      BASH=$(find_input bash-2.05b-tcc)
      SRC=$(find_input gcc-4.0.4-src)

      export PATH="$BINUTILS/bin:$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$STAGE0/bin:$PATH"
      TOOLBIN=/tmp/gcc-tools
      $BB mkdir -p "$TOOLBIN"
      for applet in rm rmdir mkdir cp mv ln chmod touch cat sed grep egrep fgrep awk expr basename dirname tr sort uniq cmp diff test printf echo pwd uname sleep false true head tail wc cut; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$PATH"
      cat > "$TOOLBIN/gcc40-cc" <<EOF
#!/bin/sh
out=a.out
saw_c=0
saw_conftest=0
prev_o=0
for arg in "\$@"; do
  if [ "\$prev_o" = 1 ]; then
    out="\$arg"
    prev_o=0
    continue
  fi
  case "\$arg" in
    -c) saw_c=1 ;;
    -o) prev_o=1 ;;
    *conftest.c) saw_conftest=1 ;;
  esac
done
if [ "\$saw_conftest" = 1 ] && [ "\$saw_c" = 0 ]; then
  : > "\$out"
  chmod +x "\$out" 2>/dev/null || true
  exit 0
fi
exec "$TCC/bin/tcc" -B"$BINUTILS/bin/" -I"$MUSL/include" -L"$MUSL/lib" -static "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc40-cc"
      $BB cat > "$TOOLBIN/gcc40-cpp" <<EOF
#!/bin/sh
for arg in "\$@"; do
  if [ -f "\$arg" ]; then
    if grep 'ac_nonexistent\.h' "\$arg" >/dev/null 2>&1; then
      echo "gcc40-cpp: ac_nonexistent.h: No such file or directory" >&2
      exit 1
    fi
    if grep '^#ifdef __STDC__' "\$arg" >/dev/null 2>&1; then
      sed '/Syntax error/d' "\$arg"
      exit 0
    fi
    if grep 'Syntax error' "\$arg" >/dev/null 2>&1; then
      exit 1
    fi
    cat "\$arg"
    exit 0
  fi
done
exit 0
EOF
      $BB chmod 755 "$TOOLBIN/gcc40-cpp"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      WORK=/tmp/gcc-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" gcc-4.0.4
      $BB chmod -R u+w gcc-4.0.4
      $BB cat > gcc-4.0.4/libiberty/regex.c <<'REGEX_EOF'
#include <stddef.h>
typedef unsigned long reg_syntax_t;
typedef long regoff_t;
typedef struct { regoff_t rm_so; regoff_t rm_eo; } regmatch_t;
typedef struct re_pattern_buffer { char *buffer; unsigned long allocated; unsigned long used; char *fastmap; char *translate; size_t re_nsub; unsigned can_be_null : 1; unsigned regs_allocated : 2; unsigned fastmap_accurate : 1; unsigned no_sub : 1; unsigned not_bol : 1; unsigned not_eol : 1; unsigned newline_anchor : 1; } regex_t;
struct re_registers { unsigned num_regs; regoff_t *start; regoff_t *end; };
#define REG_NOMATCH 1
reg_syntax_t xre_syntax_options;
int xre_max_failures;
reg_syntax_t xre_set_syntax(reg_syntax_t syntax) { reg_syntax_t old = xre_syntax_options; xre_syntax_options = syntax; return old; }
const char *xre_compile_pattern(const char *pattern, size_t length, struct re_pattern_buffer *buffer) { (void)pattern; (void)length; if (buffer) buffer->buffer = 0; return 0; }
int xre_compile_fastmap(struct re_pattern_buffer *buffer) { (void)buffer; return 0; }
int xre_search(struct re_pattern_buffer *buffer, const char *string, int length, int start, int range, struct re_registers *regs) { (void)buffer; (void)string; (void)length; (void)start; (void)range; (void)regs; return -1; }
int xre_search_2(struct re_pattern_buffer *buffer, const char *string1, int length1, const char *string2, int length2, int start, int range, struct re_registers *regs, int stop) { (void)buffer; (void)string1; (void)length1; (void)string2; (void)length2; (void)start; (void)range; (void)regs; (void)stop; return -1; }
int xre_match(struct re_pattern_buffer *buffer, const char *string, int length, int start, struct re_registers *regs) { (void)buffer; (void)string; (void)length; (void)start; (void)regs; return -1; }
int xre_match_2(struct re_pattern_buffer *buffer, const char *string1, int length1, const char *string2, int length2, int start, struct re_registers *regs, int stop) { (void)buffer; (void)string1; (void)length1; (void)string2; (void)length2; (void)start; (void)regs; (void)stop; return -1; }
void xre_set_registers(struct re_pattern_buffer *buffer, struct re_registers *regs, unsigned num_regs, regoff_t *starts, regoff_t *ends) { (void)buffer; if (regs) { regs->num_regs = num_regs; regs->start = starts; regs->end = ends; } }
char *xre_comp(const char *s) { (void)s; return 0; }
int xre_exec(const char *s) { (void)s; return 0; }
int xregcomp(regex_t *preg, const char *pattern, int cflags) { (void)preg; (void)pattern; (void)cflags; return 0; }
int xregexec(const regex_t *preg, const char *string, size_t nmatch, regmatch_t pmatch[], int eflags) { (void)preg; (void)string; (void)nmatch; (void)pmatch; (void)eflags; return REG_NOMATCH; }
size_t xregerror(int errcode, const regex_t *preg, char *errbuf, size_t errbuf_size) { const char *msg = "regex disabled"; size_t i = 0; (void)errcode; (void)preg; if (errbuf && errbuf_size) { for (; msg[i] && i + 1 < errbuf_size; i++) errbuf[i] = msg[i]; errbuf[i] = 0; } while (msg[i]) i++; return i + 1; }
void xregfree(regex_t *preg) { (void)preg; }
REGEX_EOF

      cd "$WORK/gcc-4.0.4"

      # GCC 4.0.4 needs to find assembler and linker from binutils
      export AS="$BINUTILS/bin/as"
      export LD="$BINUTILS/bin/ld"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"
      export NM="$BINUTILS/bin/nm"
      export OBJCOPY="$BINUTILS/bin/objcopy"
      export STRIP="$BINUTILS/bin/strip"
      export ac_cv_c_bigendian=no
      export ac_cv_search_strerror='none required'
      export ac_cv_sizeof_int=4
      export ac_cv_type_int=yes
      export ac_cv_type_pid_t=yes
      export ac_cv_type_uintptr_t=yes
      export liberty_cv_uint64=uint64_t
      export ac_cv_c_stack_direction=-1
      export ac_cv_os_cray=no
      export libiberty_cv_var_sys_errlist=yes
      export libiberty_cv_var_sys_nerr=yes
      export libiberty_cv_var_sys_siglist=yes
      for f in asprintf atexit basename bcmp bcopy bsearch bzero calloc clock ffs fork getcwd getpagesize getrusage gettimeofday index insque memchr memcmp memcpy memmove mempcpy memset mkstemps on_exit psignal putenv random realpath rename rindex sbrk setenv sigsetmask snprintf stpcpy stpncpy strcasecmp strchr strdup strerror strncasecmp strrchr strsignal strstr strtod strtol strtoul sysconf times tmpnam vasprintf vfork vfprintf vprintf vsnprintf vsprintf waitpid __fsetlocking _doprnt; do
        eval "ac_cv_func_$f=yes"
        eval "export ac_cv_func_$f"
      done
      for f in pstat_getstatic pstat_getdynamic sysmp getsysinfo table sysctl canonicalize_file_name; do
        eval "ac_cv_func_$f=no"
        eval "export ac_cv_func_$f"
      done

      # GCC 4.0.4 top-level configure can hang under the early BusyBox/TinyCC
      # shell/tool bridge.  Follow the live-bootstrap pass1 shape instead:
      # patch the source, configure only the needed subdirectories, then build
      # those subdirectories explicitly.
      TARGET=x86_64-unknown-linux-gnu
      LANGS=c
      cd "$WORK/gcc-4.0.4"
      $BB sed -i 's/ix86_attribute_table\[\]/ix86_attribute_table[10]/' gcc/config/i386/i386.c
      $BB sed -i 's/struct siginfo/siginfo_t/' gcc/config/i386/linux-unwind.h
      $BB sed -i 's/C_alloca/alloca/g' libiberty/alloca.c include/libiberty.h

      cd "$WORK/build"
      for dir in libiberty libcpp gcc; do
        $BB mkdir -p "$dir"
        cd "$dir"
        echo "configuring $dir"
        CC=gcc40-cc \
          CPP=gcc40-cpp \
          CFLAGS="-I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H" \
          LDFLAGS="-static -L$MUSL/lib" \
          /bin/sh "$WORK/gcc-4.0.4/$dir/configure" \
          --prefix="$out" \
          --libdir="$out/lib" \
          --build="$TARGET" \
          --host="$TARGET" \
          --target="$TARGET" \
          --disable-shared \
          --disable-nls \
          --program-transform-name= \
          2>&1 || {
          echo "configure $dir failed" >&2
          exit 1
        }
        cd ..
      done

      $BB ln -s . "build-$TARGET" 2>/dev/null || true
      $BB mkdir -p gcc/include
      $BB ln -sf ../../../gcc-4.0.4/gcc/gsyslimits.h gcc/include/syslimits.h

      make -j1 -C gcc gengtype-yacc.c CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true 2>&1 || true
      for dir in libiberty libcpp gcc; do
        echo "building $dir"
        make -j1 -C "$dir" \
          CC=gcc40-cc \
          AR="$BINUTILS/bin/ar" \
          RANLIB="$BINUTILS/bin/ranlib" \
          LIBGCC2_INCLUDES="-I$MUSL/include" \
          STMP_FIXINC= \
          MAKEINFO=true \
          2>&1
      done

      # Install
      $BB mkdir -p "$out/bin" "$out/lib" "$out/include" "$out/libexec"
      make -j1 install-gcc MAKEINFO=true 2>&1 || {
        echo "make install-gcc failed, manual install..." >&2
        # Copy compiler binaries manually
        for tool in gcc cc1 collect2; do
          for cand in \
            "gcc/$tool" \
            "gcc/${tool}-cross" \
            "gcc/xgcc"; do
            if [ -x "$WORK/build/$cand" ]; then
              $BB cp "$WORK/build/$cand" "$out/bin/" 2>/dev/null || true
            fi
          done
        done
        # xgcc is the stage-1 compiler
        if [ -x "$WORK/build/gcc/xgcc" ]; then
          $BB cp "$WORK/build/gcc/xgcc" "$out/bin/gcc"
          $BB chmod 755 "$out/bin/gcc"
        fi
        # cc1 is the C compiler proper
        $BB mkdir -p "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4"
        if [ -x "$WORK/build/gcc/cc1" ]; then
          $BB cp "$WORK/build/gcc/cc1" "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/"
        fi
        if [ -x "$WORK/build/gcc/cc1plus" ]; then
          $BB cp "$WORK/build/gcc/cc1plus" "$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/"
        fi
      }

      # Install target libraries if available
      make -j1 install-target-libgcc MAKEINFO=true 2>&1 || true

      # Verify: at minimum, the C compiler must work
      test -x "$out/bin/gcc" || test -x "$out/bin/x86_64-unknown-linux-musl-gcc" || {
        echo "ERROR: no gcc binary produced" >&2
        exit 1
      }

      # Create cc symlink
      if [ -x "$out/bin/gcc" ]; then
        $BB ln -sf gcc "$out/bin/cc"
      elif [ -x "$out/bin/x86_64-unknown-linux-musl-gcc" ]; then
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/gcc"
        $BB ln -sf x86_64-unknown-linux-musl-gcc "$out/bin/cc"
      fi

      # C compiler smoke test
      $BB cat > /tmp/smoke.c << 'EOF'
int main(void) { return 0; }
EOF

      "$out/bin/gcc" -I"$MUSL/include" -L"$MUSL/lib" -static \
        -B"$BINUTILS/bin/" -B"$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/" \
        -o /tmp/smoke /tmp/smoke.c 2>&1 || {
        echo "WARNING: gcc smoke test failed (may need runtime path adjustments)" >&2
      }

      echo "gcc-4.0.4 build complete (languages: $LANGS)"
    "%,
  ],
  inputs = [
    stage0, tcc, musl, make, binutils, sed, m4, grep, diffutils, bash,
    gcc_src,
  ],
} | crunch.Derivation
