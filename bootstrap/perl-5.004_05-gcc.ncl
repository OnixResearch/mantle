# Perl 5.004_05 rebuilt with the source-built Perl 5.003 generator and GCC.
#
# Uses explicit parser/header regeneration and a bounded miniperl source set;
# the old ignored-error/TinyCC fallback is not used. Inputs are source-derived;
# normalized-provider admission remains a separate closure-wide decision.

let crunch = import "lib.ncl" in
let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-musl-cxx.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gcc.ncl" in
let bison = crunch.derivationFile "bison-2.3-gcc.ncl" in
let m4 = crunch.derivationFile "m4-1.4.7-gcc.ncl" in
let perl_old = crunch.derivationFile "perl-5.003-gcc.ncl" in
let grep = crunch.derivationFile "grep-2.4-gcc.ncl" in

let perl_src = crunch.fetchTarball {
  url = "https://www.cpan.org/src/5.0/perl5.004_05.tar.gz",
  hash = "sha256-CNBC8fYPVU3MUDM7po08S3oRRpwNTsxUBYgqHMWjyFs=",
  name = "perl-5.004_05-src",
} in

{
  name = "perl-5.004_05-gcc-v5",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    VERSION=5.004_05
    MEMORY_ALIGNMENT_BYTES=8
    BYTE_ORDER_HEX=0x4321
    INTEGER_SIZE_BYTES=4
    RANDOM_BITS=31
    RUNTIME_LEFT=20
    RUNTIME_RIGHT=22
    EXPECTED_RUNTIME_SUM=42
    MINIPERL_SOURCES="av deb doio doop dump globals gv hv mg miniperlmain op perl perlio perly pp pp_ctl pp_hot pp_sys regcomp regexec run scope sv taint toke universal util"

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    GCC=$(find_input gcc-4.0.4-musl-cxx-v8)
    MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
    BISON=$(find_input bison-2.3-gcc-v9)
    M4=$(find_input m4-1.4.7-gcc-v5)
    PERL_OLD=$(find_input perl-5.003-gcc-v6)
    STAGE0=$(find_input stage0-posix)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input perl-5.004_05-src)

    WORK=/tmp/perl-5.004_05-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN"
    for applet in basename cat chmod cmp cp cut dirname echo expr false find head ln ls mkdir mv od printf pwd rm rmdir sed sh sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    SED="$TOOLBIN"
    CC="$TOOLBIN/gcc"
    $BB cat > "$CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
exec "$GCC/bin/gcc" -static "\$@"
EOF
    $BB chmod 755 "$CC"
    export PATH="$TOOLBIN:$BISON/bin:$M4/bin:$PERL_OLD/bin:$GREP/bin:$BINUTILS/bin:$STAGE0/bin:$PATH"
    export M4="$M4/bin/m4"
    export BISON_PKGDATADIR="$BISON/share/bison"

    PERL_AV=$($BB find "$SRC" -type f -name av.c | $BB head -n 1)
    test -n "$PERL_AV" || { echo "ERROR: Perl 5.004_05 source root missing av.c" >&2; exit 1; }
    PERL_ROOT=$($BB dirname "$PERL_AV")
    $BB mkdir -p "$WORK/src"
    $BB cp -r "$PERL_ROOT/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    $BB cat > config.h <<EOF
#define MEM_ALIGNBYTES $MEMORY_ALIGNMENT_BYTES
#define BIN "$out/bin"
#define BIN_EXP "$out/bin"
#define ARCHNAME "x86_64-linux"
#define BYTEORDER $BYTE_ORDER_HEX
#define CPPSTDIN "$CC -E"
#define CPPMINUS "-"
#define HAS_ALARM
#define HAS_FORK
#define HAS_MKDIR
#define HAS_PAUSE
#define HAS_RMDIR
#define HAS_STRERROR
#define HAS_SYS_ERRLIST
#define Strerror(e) strerror(e)
#define HAS_SYSCALL
#define HAS_TIMES
#define HAS_VPRINTF
#define Gid_t gid_t
#define I_DIRENT
#define Direntry_t struct dirent
#define I_DLFCN
#define I_FCNTL
#define I_GRP
#define I_LIMITS
#define I_MATH
#define I_NETINET_IN
#define I_PWD
#define I_STDDEF
#define I_STDLIB
#define I_STRING
#define I_SYS_DIR
#define I_SYS_IOCTL
#define I_SYS_PARAM
#define I_SYS_SELECT
#define I_SYS_STAT
#define I_SYS_TIMES
#define I_TERMIOS
#define I_TIME
#define I_UNISTD
#define I_UTIME
#define I_STDARG
#define INTSIZE $INTEGER_SIZE_BYTES
#define Off_t off_t
#define PRIVLIB "$out/lib/perl5/$VERSION"
#define _(args) args
#define RANDBITS $RANDOM_BITS
#define SCRIPTDIR "$out/bin"
#define SIG_NAME "ZERO","HUP","INT","QUIT","ILL","TRAP","IOT","BUS","FPE","KILL","USR1","SEGV","USR2","PIPE","ALRM","TERM","STKFLT","CHLD","CONT","STOP","TSTP","TTIN","TTOU","URG","XCPU","XFSZ","VTALRM","PROF","WINCH", "IO", "POLL", "PWR", "SYS", "UNUSED"
#define SIG_NUM 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
#define Size_t size_t
#define Malloc_t void *
#define Free_t void
#define SH_PATH "/bin/sh"
#define STDCHAR char
#define Uid_t uid_t
#define LOC_SED "$SED/sed"
#define ARCHLIB "$out/lib/perl5/$VERSION/"
#define CAT2(a,b) a##b
#define Gconvert(x,n,t,b) gcvt((x),(n),(b))
#define Time_t time_t
#define SSize_t ssize_t
#define Sigjmp_buf sigjmp_buf
#define Sigsetjmp(buf,save_mask) setjmp(buf)
#define Siglongjmp(buf,retval) longjmp(buf,retval)
#define Signal_t void
#define OSNAME "linux"
#undef __DATE__
EOF

    $BB rm -f perly.c perly.h perly.tab.c perly.tab.h
    "$BISON/bin/bison" -d perly.y
    test -s perly.tab.c
    test -s perly.tab.h
    $BB mv perly.tab.c perly.c
    $BB mv perly.tab.h perly.h

    for generated_name in embed keywords opcode; do
      $BB rm -f "$generated_name.h"
      "$PERL_OLD/bin/perl" "$generated_name.pl"
      test -s "$generated_name.h"
    done

    CFLAGS="-std=gnu89 -O2 -fno-common -I. -I$MUSL/include -DPERL_CORE -DPRIVLIB_EXP=\"$out/lib/perl5/$VERSION\""
    OBJECTS=
    for source_name in $MINIPERL_SOURCES; do
      "$CC" $CFLAGS -c "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      OBJECTS="$OBJECTS $source_name.o"
    done
    test -n "$OBJECTS"
    "$CC" $CFLAGS $OBJECTS -lm -o miniperl
    test -s miniperl
    $BB chmod 755 miniperl

    $BB mkdir -p "$out/bin" "$out/lib/perl5/$VERSION"
    $BB cp miniperl "$out/bin/perl"
    $BB cp miniperl "$out/bin/miniperl"
    $BB cp -r lib/. "$out/lib/perl5/$VERSION/"
    $BB chmod 755 "$out/bin/perl" "$out/bin/miniperl"

    "$out/bin/perl" -v > "$WORK/version.txt"
    $BB grep -q 'version 5.004_05' "$WORK/version.txt"
    runtime_output=$("$out/bin/perl" -e "print(($RUNTIME_LEFT+$RUNTIME_RIGHT)==$EXPECTED_RUNTIME_SUM ? \"perl500405-gcc-ok\\n\" : \"bad\\n\")")
    test "$runtime_output" = perl500405-gcc-ok
    if "$out/bin/perl" -e 'sub {' > "$WORK/rejected.out" 2> "$WORK/rejected.err"; then
      echo "ERROR: Perl 5.003 accepted malformed source" >&2
      exit 1
    fi
    test ! -s "$WORK/rejected.out"
    "$BINUTILS/bin/readelf" -h "$out/bin/perl" > "$WORK/readelf.txt"
    $BB grep -q 'ELF64' "$WORK/readelf.txt"

    echo 'Perl 5.004_05 GCC regeneration and runtime checks passed'
  "%],
  inputs = [stage0, gcc, musl, binutils, bison, m4, perl_old, grep, perl_src],
} | crunch.Derivation
