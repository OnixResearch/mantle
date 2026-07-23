# GCC-built Heirloom yacc/lex bridge used only to regenerate the first Flex
# release from authenticated grammar and scanner sources.

let crunch = import "lib.ncl" in
let heirloom_src = crunch.fetchTarball {
  url = "http://downloads.sourceforge.net/project/heirloom/heirloom-devtools/070527/heirloom-devtools-070527.tar.bz2",
  hash = "sha256-yJ2pXfdovrJ9VwR1kq0Op5J8bRq8zXMohk8GCRfX7ZY=",
  name = "heirloom-devtools-070527-src",
} in

{
  name = "heirloom-devtools-070527-gcc-v6",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox

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
    SRC=$(find_input heirloom-devtools-070527-src)

    WORK=/tmp/heirloom-devtools-gcc
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
    if test "$#" = 1 && test -d "$1"; then
      cd "$1"
    else
      cd "$WORK/src"
    fi
    SOURCE_ROOT=$PWD

    $BB mkdir -p "$out/bin" "$out/lib" "$out/lib/lex"
    $BB cp "$SOURCE_ROOT/yacc/yaccpar" "$out/lib/yaccpar"
    $BB cp "$SOURCE_ROOT/lex/ncform" "$out/lib/lex/ncform"
    $BB cp "$SOURCE_ROOT/lex/nceucform" "$out/lib/lex/nceucform"
    $BB cp "$SOURCE_ROOT/lex/nrform" "$out/lib/lex/nrform"

    cd "$SOURCE_ROOT/yacc"
    yacc_objects=
    for source_name in y1 y2 y3 y4 y5 getopt; do
      "$CC" -O2 -DPARSER=\"$out/lib/yaccpar\" -c "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      yacc_objects="$yacc_objects $source_name.o"
    done
    test -n "$yacc_objects"
    "$CC" $yacc_objects -o yacc
    test -x yacc

    cd "$SOURCE_ROOT/lex"
    "$SOURCE_ROOT/yacc/yacc" -P "$out/lib/yaccpar" parser.y
    test -s y.tab.c
    $BB mv y.tab.c parser.c
    lex_objects=
    for source_name in main sub1 sub2 sub3 header wcio parser getopt lsearch; do
      "$CC" -O2 -DFORMPATH=\"$out/lib/lex\" -c "$source_name.c" -o "$source_name.o"
      test -s "$source_name.o"
      lex_objects="$lex_objects $source_name.o"
    done
    test -n "$lex_objects"
    "$CC" $lex_objects -o lex
    test -x lex

    $BB cp "$SOURCE_ROOT/yacc/yacc" "$out/bin/yacc"
    $BB cp "$SOURCE_ROOT/lex/lex" "$out/bin/lex"
    $BB chmod 755 "$out/bin/yacc" "$out/bin/lex"

    POSITIVE_YACC="$WORK/positive.y"
    $BB cat > "$POSITIVE_YACC" <<'EOF'
%token MANTLE_TOKEN
%%
input: MANTLE_TOKEN { printf("yacc-ok\n"); } ;
%%
int yylex(void) { static int emitted = 0; if (emitted) return 0; emitted = 1; return MANTLE_TOKEN; }
int yyerror(char *message) { return message == 0; }
int main(void) { return yyparse(); }
EOF
    cd "$WORK"
    "$out/bin/yacc" positive.y
    test -s y.tab.c
    "$CC" -O2 y.tab.c -o yacc-positive
    yacc_output=$(./yacc-positive)
    test "$yacc_output" = yacc-ok

    $BB cat > "$WORK/negative.y" <<'EOF'
%token BROKEN
input: BROKEN ;
EOF
    $BB rm -f y.tab.c
    if "$out/bin/yacc" negative.y > "$WORK/yacc-negative.out" 2> "$WORK/yacc-negative.err"; then
      echo 'ERROR: heirloom yacc accepted a malformed grammar' >&2
      exit 1
    fi
    test ! -s y.tab.c
    test -s "$WORK/yacc-negative.err"

    $BB cat > "$WORK/positive.l" <<'EOF'
%%
hello  { printf("lex-ok\n"); }
[ \t\n]+ ;
. ;
%%
int yywrap(void) { return 1; }
int main(void) { return yylex(); }
EOF
    "$out/bin/lex" positive.l
    test -s lex.yy.c
    "$CC" -O2 lex.yy.c -o lex-positive
    set +e
    $BB printf '%s\n' hello | ./lex-positive > "$WORK/lex-positive.out"
    lex_status=$?
    set -e
    $BB cat "$WORK/lex-positive.out"
    echo "heirloom-lex-positive-status=$lex_status"
    test "$lex_status" = 0
    test -s "$WORK/lex-positive.out"
    $BB grep -q '^lex-ok$' "$WORK/lex-positive.out"

    $BB cat > "$WORK/negative.l" <<'EOF'
%%
[unterminated ECHO;
%%
EOF
    $BB rm -f lex.yy.c
    if "$out/bin/lex" negative.l > "$WORK/lex-negative.out" 2> "$WORK/lex-negative.err"; then
      echo 'ERROR: heirloom lex accepted a malformed scanner' >&2
      exit 1
    fi
    test -s "$WORK/lex-negative.err"

    echo 'Heirloom yacc/lex GCC generation and rejection checks passed; bootstrap bridge only'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    heirloom_src,
  ],
} | crunch.Derivation
