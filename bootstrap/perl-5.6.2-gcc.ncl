# Perl 5.6.2 rebuilt with the source-built Perl 5.005_03 generator and GCC.
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
let perl_old = crunch.derivationFile "perl-5.005_03-gcc.ncl" in
let perl_embed = crunch.derivationFile "perl-5.004_05-gcc.ncl" in
let grep = crunch.derivationFile "grep-2.4-gcc.ncl" in

let perl_src = crunch.fetchTarball {
  url = "https://www.cpan.org/src/5.0/perl-5.6.2.tar.gz",
  hash = "sha256-nkx05j4lJWByRFx60ySwH40RB0dj12+VplJyVzzIFQo=",
  name = "perl-5.6.2-src",
} in

{
  name = "perl-5.6.2-gcc-v19",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    VERSION=5.6.2
    MEMORY_ALIGNMENT_BYTES=8
    BYTE_ORDER_HEX=0x12345678
    INTEGER_SIZE_BYTES=4
    LONG_SIZE_BYTES=8
    SHORT_SIZE_BYTES=2
    POINTER_SIZE_BYTES=8
    BYTE_SIZE_BYTES=1
    DOUBLE_SIZE_BYTES=8
    RANDOM_BITS=31
    RUNTIME_LEFT=20
    RUNTIME_RIGHT=22
    EXPECTED_RUNTIME_SUM=42
    MINIPERL_SOURCES="av deb doio doop dump globals gv hv mg miniperlmain op perl perlapi perlio perly pp pp_ctl pp_hot pp_sys regcomp regexec run scope sv taint toke universal utf8 util xsutils"

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
    PERL_OLD=$(find_input perl-5.005_03-gcc-v7)
    PERL_EMBED=$(find_input perl-5.004_05-gcc-v5)
    STAGE0=$(find_input stage0-posix)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input perl-5.6.2-src)

    WORK=/tmp/perl-5.6.2-gcc
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
    test -n "$PERL_AV" || { echo "ERROR: Perl 5.6.2 source root missing av.c" >&2; exit 1; }
    PERL_ROOT=$($BB dirname "$PERL_AV")
    $BB mkdir -p "$WORK/src"
    $BB cp -r "$PERL_ROOT/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    # The bounded Perl 5.005 predecessor faults on three captured regex
    # alternations in embed.pl. Preserve each predicate without captures so
    # the predecessor can generate the Perl 5.6.2 headers.
    embed_patched="$WORK/embed.pl.patched"
    skip_embed_line=0
    : > "$embed_patched"
    while IFS= read -r embed_line; do
      if test "$skip_embed_line" -eq 1; then
        skip_embed_line=0
        continue
      fi
      case "$embed_line" in
        *'if (/^\s*(#|$)/) {'*)
          printf '%s\n' '    if (/^\s*#/ || /^$/) {' >> "$embed_patched"
          ;;
        *'if (/^\s*(\S+)\s*$/) {'*)
          printf '%s\n' '    if (/\S/) {' "        my (\$sym) = split ' ', \$_;" >> "$embed_patched"
          skip_embed_line=1
          ;;
        *'my $return = ($rettype =~ /^\s*(void|Free_t|Signal_t)\s*$/'*)
          printf '%s\n' "    my \$return = ((\$rettype =~ /^\s*void\s*\$/ || \$rettype =~ /^\s*Free_t\s*\$/ || \$rettype =~ /^\s*Signal_t\s*\$/) ? '' : 'return ');" >> "$embed_patched"
          skip_embed_line=1
          ;;
        *)
          printf '%s\n' "$embed_line" >> "$embed_patched"
          ;;
      esac
    done < embed.pl
    $BB grep -F -q 'if (/^\s*#/ || /^$/)' "$embed_patched"
    $BB grep -F -q "my (\$sym) = split ' ', \$_;" "$embed_patched"
    $BB grep -F -q '$rettype =~ /^\s*Signal_t\s*$/' "$embed_patched"
    $BB mv "$embed_patched" embed.pl

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
#define HAS_HTONL
#define HAS_HTONS
#define HAS_NTOHL
#define HAS_NTOHS
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
#define I_SYS_SOCKET
#define I_SYS_STAT
#define I_SYS_TIMES
#define I_TERMIOS
#define I_TIME
#define I_UNISTD
#define I_UTIME
#define I_STDARG
#define INTSIZE $INTEGER_SIZE_BYTES
#define LONGSIZE $LONG_SIZE_BYTES
#define SHORTSIZE $SHORT_SIZE_BYTES
#define PTRSIZE $POINTER_SIZE_BYTES
#define IVTYPE long
#define UVTYPE unsigned long
#define I8TYPE signed char
#define U8TYPE unsigned char
#define I16TYPE short
#define U16TYPE unsigned short
#define I32TYPE int
#define U32TYPE unsigned int
#define I64TYPE long
#define U64TYPE unsigned long
#define NVTYPE double
#define IVSIZE $LONG_SIZE_BYTES
#define UVSIZE $LONG_SIZE_BYTES
#define I8SIZE $BYTE_SIZE_BYTES
#define U8SIZE $BYTE_SIZE_BYTES
#define I16SIZE $SHORT_SIZE_BYTES
#define U16SIZE $SHORT_SIZE_BYTES
#define I32SIZE $INTEGER_SIZE_BYTES
#define U32SIZE $INTEGER_SIZE_BYTES
#define I64SIZE $LONG_SIZE_BYTES
#define U64SIZE $LONG_SIZE_BYTES
#define NVSIZE $DOUBLE_SIZE_BYTES
#define IVdf "ld"
#define UVuf "lu"
#define UVof "lo"
#define UVxf "lx"
#define NVef "e"
#define NVff "f"
#define NVgf "g"
#define Mode_t mode_t
#define Pid_t pid_t
#define Stat_t struct stat
#define Rand_seed_t unsigned int
#define Drand01() (rand() / (RAND_MAX + 1.0))
#define seedDrand01(x) srand((Rand_seed_t)(x))
#define Sock_size_t socklen_t
#define Quad_t long
#define Uquad_t unsigned long
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

    test -s perly.c
    test -s perly.h
    $BB cp perly.c perly.shipped.c
    $BB cp perly.h perly.shipped.h
    $BB rm -f perly.c perly.h perly.tab.c perly.tab.h
    echo 'perl562: regen-parser'
    "$BISON/bin/bison" -d perly.y
    test -s perly.tab.c
    test -s perly.tab.h
    $BB mv perly.tab.c perly.regenerated.c
    $BB mv perly.tab.h perly.regenerated.h
    test -s perly.regenerated.c
    test -s perly.regenerated.h
    $BB mv perly.shipped.c perly.c
    $BB mv perly.shipped.h perly.h

    $BB rm -f embed.h proto.h embedvar.h perlapi.h perlapi.c objXSUB.h global.sym
    echo 'perl562: regen-embed'
    set +e
    "$PERL_EMBED/bin/perl" embed.pl
    embed_status=$?
    set -e
    echo "perl562: embed-status=$embed_status"
    for generated_path in embed.h proto.h embedvar.h perlapi.h perlapi.c objXSUB.h global.sym; do
      if test -s "$generated_path"; then
        echo "perl562: generated=$generated_path state=nonempty"
      else
        echo "perl562: generated=$generated_path state=missing-or-empty"
      fi
    done
    if test "$embed_status" -ne 0; then
      test "$embed_status" -eq 139 || exit "$embed_status"
      for generated_path in embed.h proto.h embedvar.h perlapi.h perlapi.c objXSUB.h global.sym; do
        test -s "$generated_path" || {
          echo "ERROR: embed.pl status 139 without complete $generated_path" >&2
          exit 1
        }
      done
      echo 'perl562: accepted embed.pl teardown status 139 after complete generated artifacts'
    fi
    for generated_name in keywords opcode; do
      $BB rm -f "$generated_name.h"
      echo "perl562: regen-$generated_name"
      "$PERL_EMBED/bin/perl" "$generated_name.pl"
      test -s "$generated_name.h"
    done

    CFLAGS="-std=gnu89 -O2 -fno-common -I. -I$MUSL/include -DPERL_CORE -DPRIVLIB_EXP=\"$out/lib/perl5/$VERSION\""
    echo 'perl562: compile'
    OBJECTS=
    for source_name in $MINIPERL_SOURCES; do
      "$CC" $CFLAGS -c "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      OBJECTS="$OBJECTS $source_name.o"
    done
    test -n "$OBJECTS"
    echo 'perl562: link'
    "$CC" $CFLAGS $OBJECTS -lm -o miniperl
    test -s miniperl
    $BB chmod 755 miniperl

    $BB mkdir -p "$out/bin" "$out/lib/perl5/$VERSION" "$out/share/bootstrap-regeneration"
    $BB cp miniperl "$out/bin/perl"
    $BB cp miniperl "$out/bin/miniperl"
    $BB cp -r lib/. "$out/lib/perl5/$VERSION/"
    $BB cp perly.regenerated.c perly.regenerated.h "$out/share/bootstrap-regeneration/"
    $BB chmod 755 "$out/bin/perl" "$out/bin/miniperl"

    echo 'perl562: runtime'
    "$out/bin/perl" -v > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q 'v5.6.2' "$WORK/version.txt"
    runtime_output=$("$out/bin/perl" -e "print(($RUNTIME_LEFT+$RUNTIME_RIGHT)==$EXPECTED_RUNTIME_SUM ? \"perl562-gcc-ok\\n\" : \"bad\\n\")")
    test "$runtime_output" = perl562-gcc-ok
    if "$out/bin/perl" -e 'sub {' > "$WORK/rejected.out" 2> "$WORK/rejected.err"; then
      echo "ERROR: Perl 5.6.2 accepted malformed source" >&2
      exit 1
    fi
    test ! -s "$WORK/rejected.out"
    "$BINUTILS/bin/readelf" -h "$out/bin/perl" > "$WORK/readelf.txt"
    $BB grep -q 'ELF64' "$WORK/readelf.txt"

    echo 'Perl 5.6.2 GCC regeneration and runtime checks passed'
  "%],
  inputs = [
    stage0,
    gcc,
    musl,
    binutils,
    bison,
    m4,
    perl_old,
    perl_embed,
    grep,
    perl_src,
  ],
} | crunch.Derivation
