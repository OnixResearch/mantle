# Perl 5.000 rebuilt with GCC pass6 and regenerated bootstrap sources.
#
# Tracks live-bootstrap's exact miniperl source set and regeneration sequence:
# Bison perly.c/perly.h, embed_h.SH, keywords.sh, and opcode.sh. The resulting
# interpreter is runtime tested but remains a bootstrap tool, not provider proof.

let crunch = import "lib.ncl" in
let generators = import "generator-chain-gcc.ncl" in
let bison = generators.bison in
let gawk = generators.gawk in

let perl_src = crunch.fetchTarball {
  url = "https://github.com/Perl/perl5/archive/perl-5.000.tar.gz",
  hash = "sha256-s9LMsFfFf6Ws84cG/GW55znYlQBbn56U/C9TSiWczFE=",
  name = "perl-5.000-src",
} in

{
  name = "perl-5.000-gcc-v7",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    GCC_VERSION=4.0.4
    MINIPERL_SOURCES="av deb doio doop dump gv hv mg miniperlmain op perl perly pp pp_ctl pp_hot pp_sys regcomp regexec run scope sv taint toke util"
    EXPECTED_OPCODE_VALUES=340

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    GCC=$(find_input gcc-4.0.4-musl-pass4-v5)
    MUSL=$(find_input musl-1.1.24-gcc-pass4-v2)
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
    BISON=$(find_input bison-2.3-gcc-v9)
    M4=$(find_input m4-1.4.7-gcc-v5)
    STAGE0=$(find_input stage0-posix)
    SED=$(find_input sed-4.0.9-musl)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input perl-5.000-src)

    WORK=/tmp/perl-5.000-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN"
    for applet in basename cat chmod cmp cp cut dirname echo expr find head ln ls mkdir mv od printf pwd rm rmdir sh sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    CC="$TOOLBIN/gcc"
    $BB cat > "$CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
exec "$GCC/bin/gcc" "\$@"
EOF
    $BB chmod 755 "$CC"
    export PATH="$TOOLBIN:$BISON/bin:$M4/bin:$SED/bin:$GAWK/bin:$GREP/bin:$BINUTILS/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL=/bin/sh
    export SHELL=/bin/sh
    export M4="$M4/bin/m4"
    export BISON_PKGDATADIR="$BISON/share/bison"

    PERL_AV=$($BB find "$SRC" -type f -name av.c | $BB head -n 1)
    test -n "$PERL_AV" || { echo "ERROR: Perl source root missing av.c" >&2; exit 1; }
    PERL_ROOT=$($BB dirname "$PERL_AV")
    $BB mkdir -p "$WORK/src"
    $BB cp -r "$PERL_ROOT/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    $BB cat > config.h <<EOF
#define MEM_ALIGNBYTES 8
#define BIN "$out/bin"
#define BYTEORDER 0x4321
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
#define I_LIMITS
#define I_MATH
#define I_NETINET_IN
#define I_STDDEF
#define I_STDLIB
#define I_STRING
#define I_SYS_DIR
#define I_SYS_IOCTL
#define I_SYS_PARAM
#define I_SYS_SELECT
#define I_SYS_TIMES
#define I_TERMIOS
#define I_TIME
#define I_UNISTD
#define I_UTIME
#define I_STDARG
#define INTSIZE 4
#define Off_t off_t
#define PRIVLIB "$out/lib/perl5"
#define _(args) args
#define RANDBITS 31
#define SCRIPTDIR "$out/bin"
#define SIG_NAME "ZERO","HUP","INT","QUIT","ILL","TRAP","IOT","BUS","FPE","KILL","USR1","SEGV","USR2","PIPE","ALRM","TERM","STKFLT","CHLD","CONT","STOP","TSTP","TTIN","TTOU","URG","XCPU","XFSZ","VTALRM","PROF","WINCH", "IO", "POLL", "PWR", "SYS", "UNUSED"
#define Size_t size_t
#define STDCHAR char
#define Uid_t uid_t
#define LOC_SED "$SED/bin/sed"
#define ARCHLIB "$out/lib/perl5/"
#define CAT2(a,b) a##b
#define Gconvert(x,n,t,b) gcvt((x),(n),(b))
#define Time_t time_t
#define SSize_t ssize_t
EOF

    $BB rm -f perly.c perly.h perly.tab.c perly.tab.h
    "$BISON/bin/bison" -d perly.y
    test -s perly.tab.c
    test -s perly.tab.h
    $BB mv perly.tab.c perly.c
    $BB mv perly.tab.h perly.h

    $BB rm -f embed.h
    $BB chmod 755 embed_h.SH
    ./embed_h.SH
    test -s embed.h

    $BB cat > keywords.sh <<'EOF'
#!/bin/sh
set -e
sed -e '1,/__END__/ d' keywords.pl | sed '1d' | awk '{print "#define", "KEY_"$0, NR-1}' > keywords.h
EOF
    $BB chmod 755 keywords.sh
    $BB rm -f keywords.h
    ./keywords.sh
    test -s keywords.h

    $BB cat > opcode.awk <<'EOF'
BEGIN {
flag_m = 1
flag_f = 2
flag_s = 4
flag_t = 8
flag_i = 16
flag_I = 32
flag_d = 64
flag_u = 128
argument_multiplier_initial = 256
argument_multiplier_step = 16
optional_argument_offset = 8
}
{
argsum = 0
argsum += flag_m * (index($4, "m") != 0)
argsum += flag_f * (index($4, "f") != 0)
argsum += flag_s * (index($4, "s") != 0)
argsum += flag_t * (index($4, "t") != 0)
argsum += flag_i * (index($4, "i") != 0)
argsum += flag_I * (index($4, "I") != 0)
argsum += flag_d * (index($4, "d") != 0)
argsum += flag_u * (index($4, "u") != 0)
mul = argument_multiplier_initial
arg_val="SLAHCFR"
argstr=$5
gsub(" ", "", argstr)
split(argstr, args, "")
for(i=1; i<=length(argstr); i+=1) {
argnum = index(arg_val, args[i])
if (i < length(argstr) && args[i + 1] == "?") {
argnum += optional_argument_offset
i += 1
}
argsum += argnum * mul
mul *= argument_multiplier_step
}
printf("\t0x%08x,\t/* %s */\n", argsum, $1)
}
EOF
    $BB cp opcode.h "$WORK/opcode-shipped.h"
    $BB cat > opcode.sh <<'EOF'
#!/bin/sh
set -e
sed -e '1,/__END__/ d; s/[#].*$//g; /^$/d' opcode.pl | tr -s '\t' '\t' > data
exec 1> opcode.h
echo "typedef enum {"
awk '{print "\tOP_"toupper($1)","}' data
echo " OP_max"
echo "} opcode;"
echo "#define MAXO " $(wc -l data | awk '{print $1}')
printf '\n#ifndef DOINIT\nEXT char *op_name[];\n#else\nEXT char *op_name[] = {\n'
awk -F'\t' '{print "\t\""$2"\","}' data
printf '};\n#endif\n'
awk -F'\t' '{print "OP *\t"$3"\t_((OP* op));"}' data | sort | uniq
awk '{print "OP *\tpp_"$1"\t_((void));"}' data
printf '\n#ifndef DOINIT\nEXT OP * (*ppaddr[])();\n#else\nEXT OP * (*ppaddr[])() = {\n'
awk '{print "\tpp_"$1","}' data
printf '};\n#endif\n'
printf '\n#ifndef DOINIT\nEXT OP * (*check[])();\n#else\nEXT OP * (*check[])() = {\n'
awk -F'\t' '{print "\t"$3",\t/* "$1" */"}' data
printf '};\n#endif\n'
printf '\n#ifndef DOINIT\nEXT U32 opargs[];\n#else\nEXT U32 opargs[] = {\n'
awk -F'\t' -f opcode.awk data
printf '};\n#endif\n'
rm data
EOF
    $BB chmod 755 opcode.sh
    $BB rm -f opcode.h
    ./opcode.sh
    test -s opcode.h
    "$GREP/bin/grep" '0x[0-9a-fA-F]' "$WORK/opcode-shipped.h" | "$GAWK/bin/awk" '{print $1}' > "$WORK/opcode-shipped-values.txt"
    "$GREP/bin/grep" '0x[0-9a-fA-F]' opcode.h | "$GAWK/bin/awk" '{print $1}' > "$WORK/opcode-generated-values.txt"
    shipped_value_count=$($BB wc -l < "$WORK/opcode-shipped-values.txt")
    generated_value_count=$($BB wc -l < "$WORK/opcode-generated-values.txt")
    test "$shipped_value_count" = "$EXPECTED_OPCODE_VALUES"
    test "$generated_value_count" = "$EXPECTED_OPCODE_VALUES"
    $BB cmp "$WORK/opcode-shipped-values.txt" "$WORK/opcode-generated-values.txt"

    CFLAGS="-std=gnu89 -O2 -fno-common -I. -I$MUSL/include -DPERL_CORE"
    OBJECTS=
    for source_name in $MINIPERL_SOURCES; do
      "$CC" $CFLAGS -c "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      OBJECTS="$OBJECTS $source_name.o"
    done
    test -n "$OBJECTS"
    "$CC" -static $OBJECTS -lm -o miniperl
    test -s miniperl
    $BB chmod 755 miniperl

    $BB mkdir -p "$out/bin" "$out/lib/perl5"
    $BB cp miniperl "$out/bin/perl"
    $BB cp miniperl "$out/bin/miniperl"
    $BB chmod 755 "$out/bin/perl" "$out/bin/miniperl"

    perl_output=$("$out/bin/perl" -e 'my $value = 19 + 23; print $value == 42 ? "perl5000-gcc-ok\n" : "bad\n"')
    test "$perl_output" = perl5000-gcc-ok
    if "$out/bin/perl" -e 'if ('; then
      echo "ERROR: Perl accepted malformed program" >&2
      exit 1
    fi

    echo 'Perl 5.000 GCC regeneration and runtime checks passed'
  "%],
  # Bison already declares every shared compiler/runtime/generator input used
  # above. Flatten those inputs here so they remain direct sandbox inputs; Gawk
  # stays a direct tool while its overlapping build closure is not remounted.
  inputs = bison.inputs @ [bison, gawk, perl_src],
} | crunch.Derivation
