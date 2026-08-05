# GCC-built Flex 2.6.4 over the regenerated Flex 2.5.11 scanner and the
# conventional Bison/M4 bridge. This stage regenerates parser, scanner, and
# skeleton C sources before compiling them.

let crunch = import "lib.ncl" in
let flex_src = crunch.fetchTarball {
  url = "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz",
  hash = "sha256-HBPAUDpqF4HYocMD3YDIk6OaIUrZw8Gk3K59OmHB6xU=",
  name = "flex-2.6.4-src",
} in

{
  name = "flex-2.6.4-gcc-v1",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    EXPECTED_OUTPUT=token:8642

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
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
    FLEX_OLD=$(find_input flex-2.5.11-gcc-v6)
    BISON=$(find_input bison-2.3-gcc-v9)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    SRC=$(find_input flex-2.6.4-src)

    WORK=/tmp/flex-2.6.4-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$WORK/src"
    for applet in basename cat chmod cp dirname echo false grep head ln mkdir mv printf pwd rm sed sh tail test touch tr true uname wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
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
    export PATH="$TOOLBIN:$FLEX_OLD/bin:$BISON/bin:$M4TOOL/bin:$BINUTILS/bin:$STAGE0/bin"

    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
      "$BISON/bin/bison" -d -o src/parse.c src/parse.y
    test -s src/parse.c
    test -s src/parse.h
    "$FLEX_OLD/bin/flex" -o src/scan.c src/scan.l
    test -s src/scan.c
    /bin/sh src/mkskel.sh src "$M4TOOL/bin/m4" 2.6.4 > src/skel.c
    test -s src/skel.c

    $BB cat > config.h <<EOF
#define PACKAGE "flex"
#define PACKAGE_NAME "flex"
#define PACKAGE_VERSION "2.6.4"
#define VERSION "2.6.4"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_WAIT_H 1
#define STDC_HEADERS 1
#define M4 "$M4TOOL/bin/m4"
EOF

    CFLAGS="-O2 -I. -Isrc -DHAVE_CONFIG_H"
    OBJECTS=
    for source_name in buf ccl dfa ecs filter gen main misc nfa options parse regex scan scanflags scanopt skel sym tables tables_shared tblcmp yylex; do
      "$CC" -c $CFLAGS "src/$source_name.c" -o "src/$source_name.o"
      test -s "src/$source_name.o"
      OBJECTS="$OBJECTS src/$source_name.o"
    done
    test -n "$OBJECTS"
    "$CC" $OBJECTS -o flex
    test -x flex

    "$CC" -c $CFLAGS src/libmain.c -o "$WORK/libmain.o"
    "$CC" -c $CFLAGS src/libyywrap.c -o "$WORK/libyywrap.o"
    "$BINUTILS/bin/ar" cr "$WORK/libfl.a" "$WORK/libmain.o" "$WORK/libyywrap.o"
    "$BINUTILS/bin/ranlib" "$WORK/libfl.a"
    test -s "$WORK/libfl.a"

    $BB mkdir -p "$out/bin" "$out/lib" "$out/include" "$out/share/bootstrap-regeneration"
    $BB cp flex "$out/bin/flex"
    $BB chmod 755 "$out/bin/flex"
    $BB ln -s flex "$out/bin/lex"
    $BB ln -s flex "$out/bin/flex++"
    $BB cp "$WORK/libfl.a" "$out/lib/libfl.a"
    $BB cp src/FlexLexer.h "$out/include/FlexLexer.h"
    $BB cp src/parse.c src/parse.h src/scan.c src/skel.c "$out/share/bootstrap-regeneration/"

    "$out/bin/flex" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q 'flex 2.6.4' "$WORK/version.txt"

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
    scanner_output=$($BB printf '%s\n' '8642' | "$WORK/positive-scanner")
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
    echo 'Flex 2.6.4 GCC source-regeneration, scanner execution, and rejection checks passed; not provider admission'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "flex-2.5.11-gcc.ncl",
    crunch.derivationFile "bison-2.3-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    flex_src,
  ],
} | crunch.Derivation
