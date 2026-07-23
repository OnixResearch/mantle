# Self-regenerated Bison 3.4.1 rebuild with the closed GCC generator cycle.
# A conventionally built release predecessor regenerates Bison's parser and
# scanners from authenticated grammar/scanner sources before this successor is
# configured and compiled.

let crunch = import "lib.ncl" in
let bison_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/bison/bison-3.4.1.tar.xz",
  hash = "sha256-vKqyf0sdiIdtfaEILKmi1ULhs+AqVSboYSpGhgEsYsI=",
  name = "bison-3.4.1-src",
} in

{
  name = "bison-3.4.1-gcc-v2",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    BUILD_JOBS=1
    EXPECTED_RESULT=42
    PARSER_LEFT=19
    PARSER_RIGHT=23
    PERCENT=$($BB printf '\045')
    INTEGER_FORMAT="$PERCENT"d

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    STAGE0=$(find_input stage0-posix)
    GCC=$(find_input gcc-4.0.4-musl-pass6-v1)
    MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
    MAKE=$(find_input make-3.82-tcc)
    FLEX=$(find_input flex-2.6.4-gcc-v1)
    BISON_STAGE1=$(find_input bison-3.4.1-stage1-gcc-v1)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    PERL=$(find_input perl-5.6.2-configured-gcc-v16)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input bison-3.4.1-src)

    WORK=/tmp/bison-3.4.1-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$WORK/src"
    for applet in basename cat chmod cmp cp cut dirname echo expr false find head install ln ls mkdir mv printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    $BB ln -sf "$GAWK/bin/gawk" "$TOOLBIN/awk"
    $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
    $BB ln -sf "$GREP/bin/egrep" "$TOOLBIN/egrep"
    $BB ln -sf "$GREP/bin/fgrep" "$TOOLBIN/fgrep"
    CC="$TOOLBIN/gcc"
    $BB cat > "$CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \\
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \\
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \\
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \\
MANTLE_GCC_LINK_CRT="$MUSL/lib" \\
exec "$GCC/bin/gcc" \\
  -B"$GCC/bin/" \\
  -B"$GCC/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/" \\
  -B"$GCC/lib/gcc/x86_64-unknown-linux-musl/4.0.4/" \\
  -static -I"$MUSL/include" "\$@"
EOF
    $BB chmod 755 "$CC"
    export PATH="$TOOLBIN:$MAKE/bin:$FLEX/bin:$BISON_STAGE1/bin:$M4TOOL/bin:$PERL/bin:$GAWK/bin:$GREP/bin:$BINUTILS/bin:$STAGE0/bin"
    export CONFIG_SHELL=/bin/sh
    export M4="$M4TOOL/bin/m4"
    export BISON_PKGDATADIR="$BISON_STAGE1/share/bison"
    unset MAKEFLAGS MFLAGS MAKELEVEL

    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    test -s src/parse-gram.c
    test -s src/scan-code.c

    M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON_STAGE1/share/bison" \
      "$BISON_STAGE1/bin/bison" -d -o src/parse-gram.c src/parse-gram.y
    test -s src/parse-gram.c
    test -s src/parse-gram.h
    for scanner_name in code gram skel; do
      "$FLEX/bin/flex" -o "src/scan-$scanner_name.c" "src/scan-$scanner_name.l"
      test -s "src/scan-$scanner_name.c"
    done

    CC="$CC" ./configure --prefix="$out" --disable-nls CFLAGS='-O2' LDFLAGS='-static'
    "$MAKE/bin/make" -j "$BUILD_JOBS"
    "$MAKE/bin/make" -j "$BUILD_JOBS" install prefix="$out"

    test -x "$out/bin/bison"
    test -d "$out/share/bison"
    test -s "$out/share/bison/skeletons/yacc.c"
    "$out/bin/bison" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q 'bison (GNU Bison) 3.4.1' "$WORK/version.txt"

    $BB cat > "$WORK/positive.y" <<EOF
%token NUMBER
%%
input: NUMBER '+' NUMBER { printf("$INTEGER_FORMAT\\n", \$1 + \$3); } ;
%%
int yylex(void) { static int index = 0; static int tokens[] = { NUMBER, '+', NUMBER, 0 }; yylval = index == 0 ? $PARSER_LEFT : $PARSER_RIGHT; return tokens[index++]; }
int yyerror(char const *message) { return message == 0; }
int main(void) { return yyparse(); }
EOF
    M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$out/share/bison" \
      "$out/bin/bison" -o "$WORK/positive.c" "$WORK/positive.y"
    test -s "$WORK/positive.c"
    "$CC" -O2 "$WORK/positive.c" -o "$WORK/positive-parser"
    parser_output=$("$WORK/positive-parser")
    test "$parser_output" = "$EXPECTED_RESULT"

    $BB cat > "$WORK/negative.y" <<'EOF'
%token BROKEN
%%
input: BROKEN {
%%
EOF
    if M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$out/share/bison" \
      "$out/bin/bison" -o "$WORK/negative.c" "$WORK/negative.y" > "$WORK/negative.out" 2> "$WORK/negative.err"; then
      echo 'ERROR: bison accepted a malformed grammar' >&2
      exit 1
    fi
    test ! -s "$WORK/negative.c"
    test -s "$WORK/negative.err"

    "$BINUTILS/bin/readelf" -h "$out/bin/bison" > "$WORK/readelf.txt"
    $BB grep -q 'ELF64' "$WORK/readelf.txt"
    echo 'Bison 3.4.1 GCC self-regeneration, parser execution, and rejection checks passed; not provider admission'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-stage1-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    bison_src,
  ],
} | crunch.Derivation
