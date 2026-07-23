# GCC-built bootstrap Flex from a release archive carrying generated parser,
# scanner, and skeleton C sources. This stage breaks the lex self-hosting cycle;
# the following Flex stage regenerates from authenticated .l/.y inputs.

let crunch = import "lib.ncl" in
let flex_src = crunch.fetchTarball {
  url = "https://downloads.sourceforge.net/project/flex/flex-2.5.36.tar.gz",
  hash = "sha256-IqxFTFa2cKyjoELXwZkYB81WqoW9oJ53IklRff8KRBU=",
  name = "flex-2.5.36-src",
} in

{
  name = "flex-2.5.36-gcc-v4",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    BUILD_JOBS=1
    EXPECTED_OUTPUT=token:2468

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
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    SRC=$(find_input flex-2.5.36-src)

    WORK=/tmp/flex-2.5.36-gcc
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
    export PATH="$TOOLBIN:$MAKE/bin:$M4TOOL/bin:$GAWK/bin:$GREP/bin:$BINUTILS/bin:$STAGE0/bin"
    export CONFIG_SHELL=/bin/sh
    unset MAKEFLAGS MFLAGS MAKELEVEL

    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    test -s scan.c
    test -s parse.c
    test -s skel.c

    CC="$CC" ./configure --prefix="$out" --disable-shared CFLAGS='-O2' LDFLAGS='-static'
    "$MAKE/bin/make" -C lib -j "$BUILD_JOBS" libcompat.a
    "$MAKE/bin/make" -j "$BUILD_JOBS" flex libfl.a LEX=false YACC=false
    test -x flex
    test -s libfl.a
    $BB mkdir -p "$out/bin" "$out/lib" "$out/include"
    $BB cp flex "$out/bin/flex"
    $BB chmod 755 "$out/bin/flex"
    $BB ln -s flex "$out/bin/lex"
    $BB cp libfl.a "$out/lib/libfl.a"
    $BB cp FlexLexer.h "$out/include/FlexLexer.h"

    test -x "$out/bin/flex"
    test -L "$out/bin/lex"
    "$out/bin/flex" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q 'flex 2.5.36' "$WORK/version.txt"

    $BB cat > "$WORK/positive.l" <<'EOF'
%%
[0-9]+  { printf("token:%s\n", yytext); }
[ \t\n]+ ;
.       ;
%%
int yywrap(void) { return 1; }
int main(void) { return yylex(); }
EOF
    "$out/bin/flex" -o "$WORK/positive.c" "$WORK/positive.l"
    test -s "$WORK/positive.c"
    "$CC" -O2 "$WORK/positive.c" -o "$WORK/positive-scanner"
    scanner_output=$($BB printf '%s\n' '2468' | "$WORK/positive-scanner")
    test "$scanner_output" = "$EXPECTED_OUTPUT"

    $BB cat > "$WORK/negative.l" <<'EOF'
%%
[unterminated  ECHO;
%%
EOF
    if "$out/bin/flex" -o "$WORK/negative.c" "$WORK/negative.l" > "$WORK/negative.out" 2> "$WORK/negative.err"; then
      echo 'ERROR: flex accepted a malformed scanner specification' >&2
      exit 1
    fi
    test ! -s "$WORK/negative.c"
    test -s "$WORK/negative.err"

    "$BINUTILS/bin/readelf" -h "$out/bin/flex" > "$WORK/readelf.txt"
    $BB grep -q 'ELF64' "$WORK/readelf.txt"
    echo 'Flex 2.5.36 GCC release-source scanner checks passed; bootstrap bridge only'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    flex_src,
  ],
} | crunch.Derivation
