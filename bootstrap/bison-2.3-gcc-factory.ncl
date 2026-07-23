# Conventional Bison 2.3 rebuild with the closed GCC/musl/clean-binutils cycle.
#
# This replaces the earlier ignored-error TinyCC construction for regeneration
# work. Runtime admission covers parser generation, compilation, execution, and
# malformed-grammar rejection. It remains a bootstrap tool, not provider proof.

fun base m4 grep =>
let crunch = import "lib.ncl" in

let bison_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/bison/bison-2.3.tar.bz2",
  hash = "sha256-A7EYDEuY+Hw8DHostT2RBv+kjkMumqDlshfpCi2I77U=",
  name = "bison-2.3-src",
} in

{
  name = "bison-2.3-gcc-v9",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    GCC_VERSION=4.0.4
    JOB_COUNT=1

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    BASE=$(find_input gcc-generator-base-v4)
    GCC="$BASE/gcc"
    MUSL="$BASE/musl"
    BINUTILS="$BASE/binutils"
    MAKE="$BASE/make"
    STAGE0="$BASE/stage0"
    SED="$BASE/sed"
    DIFF="$BASE/diffutils"
    CONFIG_SUB="$BASE/share/automake/config.sub"
    M4=$(find_input m4-1.4.7-gcc-v5)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input bison-2.3-src)

    WORK=/tmp/bison-2.3-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN"
    for applet in awk basename cat chmod cmp cp cut dirname echo expr false find head ln ls mkdir mv od printf pwd rm rmdir sh sleep sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    CC="$TOOLBIN/gcc"
    $BB cat > "$CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \
MANTLE_GCC_LINK_CRT="$MUSL/lib" \
exec "$GCC/bin/gcc" \
  -B"$GCC/bin/" \
  -B"$GCC/libexec/gcc/$TARGET/$GCC_VERSION/" \
  -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" \
  -static "\$@"
EOF
    $BB chmod 755 "$CC"
    export PATH="$TOOLBIN:$M4/bin:$SED/bin:$GREP/bin:$DIFF/bin:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL=/bin/sh
    export SHELL=/bin/sh
    export MAKEINFO=true
    export HELP2MAN=true
    unset MAKEFLAGS MFLAGS MAKELEVEL

    $BB cp -r "$SRC" "$WORK/src"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    for config_sub in $($BB find . -name config.sub -type f); do
      $BB cp "$CONFIG_SUB" "$config_sub"
      $BB chmod 755 "$config_sub"
    done
    $BB cp "$CONFIG_SUB" build-aux/config.sub
    $BB chmod 755 build-aux/config.sub
    canonical_target=$(/bin/sh build-aux/config.sub "$TARGET")
    test "$canonical_target" = "$TARGET"
    test -f configure || { echo "ERROR: Bison configure missing" >&2; exit 1; }

    CC="$CC" \
    AR="$BINUTILS/bin/ar" \
    RANLIB="$BINUTILS/bin/ranlib" \
    M4=m4 \
    CPPFLAGS="-I$MUSL/include" \
    CFLAGS="-O2 -fno-common -I$MUSL/include" \
    LDFLAGS="-static" \
    ./configure \
      --prefix="$out" \
      --build="$TARGET" \
      --host="$TARGET" \
      --disable-nls \
      --disable-yacc

    make -j"$JOB_COUNT" \
      CC="$CC" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      M4=m4 \
      MAKEINFO=true \
      HELP2MAN=true
    make -j"$JOB_COUNT" install MAKEINFO=true HELP2MAN=true

    test -x "$out/bin/bison"
    export M4="$M4/bin/m4"
    export BISON_PKGDATADIR="$out/share/bison"
    "$out/bin/bison" --version > "$WORK/version.txt"
    $BB grep -q 'bison (GNU Bison) 2.3' "$WORK/version.txt"

    $BB cat > "$WORK/smoke.y" <<'EOF'
%token END
%%
input: ;
%%
int yylex(void) { return 0; }
void yyerror(const char *message) { (void)message; }
int main(void) { return yyparse(); }
EOF
    "$out/bin/bison" -d -o "$WORK/smoke.c" "$WORK/smoke.y"
    test -s "$WORK/smoke.c"
    test -s "$WORK/smoke.h"
    "$CC" -I"$MUSL/include" "$WORK/smoke.c" -o "$WORK/smoke"
    test -s "$WORK/smoke"
    $BB chmod 755 "$WORK/smoke"
    "$WORK/smoke"

    $BB printf '%s\n' '%%%% malformed:' > "$WORK/rejected.y"
    $BB rm -f "$WORK/rejected.c"
    if "$out/bin/bison" -o "$WORK/rejected.c" "$WORK/rejected.y"; then
      echo "ERROR: Bison accepted malformed grammar" >&2
      exit 1
    fi
    test ! -e "$WORK/rejected.c"

    echo 'Bison 2.3 GCC runtime checks passed'
  "%],
  inputs = [base, m4, grep, bison_src],
} | crunch.Derivation
