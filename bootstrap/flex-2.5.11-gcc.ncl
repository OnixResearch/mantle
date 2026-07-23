# Conventional Flex 2.5.11 rebuild with the closed GCC/musl/binutils cycle.
# The authenticated release carries generated C sources; regeneration of Flex
# itself remains a later claim, while generated consumer scanners are executed
# and malformed scanner specifications are rejected here.

let crunch = import "lib.ncl" in
let flex_src = crunch.fetchTarball {
  url = "https://github.com/westes/flex/archive/d160f0247ba1611aa59d28f027d6292ba24abb50.tar.gz",
  hash = "sha256-FZW6ydHxZ2gUpxDcAfMkwNoYp32RbpG5T2y6MuSe86Y=",
  name = "flex-2.5.11-src",
} in

{
  name = "flex-2.5.11-gcc-v6",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    EXPECTED_OUTPUT=token:12345

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
    HEIRLOOM=$(find_input heirloom-devtools-070527-gcc-v6)
    FLEX_BOOT=$(find_input flex-2.5.36-gcc-v4)
    SRC=$(find_input flex-2.5.11-src)

    WORK=/tmp/flex-2.5.11-gcc
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
    export PATH="$TOOLBIN:$BINUTILS/bin:$STAGE0/bin"

    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    set -- "$WORK/src"/*
    test "$#" = 1
    test -d "$1"
    cd "$1"
    : > config.h

    "$HEIRLOOM/bin/yacc" -P "$HEIRLOOM/lib/yaccpar" -d parse.y
    test -s y.tab.c
    test -s y.tab.h
    $BB mv y.tab.c parse.c
    $BB mv y.tab.h parse.h
    "$FLEX_BOOT/bin/flex" -o scan.c scan.l
    test -s scan.c
    /bin/sh ./mkskel.sh flex.skl > skel.c
    test -s skel.c

    CFLAGS="-O2 -I. -DHAVE_STRING_H -DHAVE_LIMITS_H -DHAVE_UNISTD_H -DSTDC_HEADERS -DVERSION=\"2.5.11\""
    OBJECTS=
    for source_name in buf ccl dfa ecs gen main misc nfa options parse scan scanopt skel sym tblcmp yylex; do
      "$CC" -c $CFLAGS "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      OBJECTS="$OBJECTS $source_name.o"
    done
    test -n "$OBJECTS"
    "$CC" $OBJECTS -o flex
    test -x flex

    $BB mkdir -p "$out/bin"
    $BB cp flex "$out/bin/flex"
    $BB chmod 755 "$out/bin/flex"
    $BB ln -s flex "$out/bin/lex"

    "$out/bin/flex" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q 'flex 2.5.11' "$WORK/version.txt"

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
    test -x "$WORK/positive-scanner"
    scanner_output=$($BB printf '%s\n' '12345' | "$WORK/positive-scanner")
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
    echo 'Flex 2.5.11 GCC scanner execution and rejection checks passed; not provider admission'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "heirloom-devtools-gcc.ncl",
    crunch.derivationFile "flex-2.5.36-gcc.ncl",
    # Flex 2.5.36 records the selected M4 path in its executable.
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    flex_src,
  ],
} | crunch.Derivation
