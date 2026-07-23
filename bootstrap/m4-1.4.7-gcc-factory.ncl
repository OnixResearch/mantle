# Conventional GNU M4 1.4.7 rebuild with the closed GCC/musl/binutils cycle.
#
# This replaces the TinyCC-era diagnostic implementation whose varargs patches
# broke frozen-state support required by Bison and Autoconf.

fun base grep =>
let crunch = import "lib.ncl" in
let m4_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/m4/m4-1.4.7.tar.bz2",
  hash = "sha256-4SO98HrHjkV6a+NazK3EatKsynMNhfWXUpi1L5SeMMc=",
  name = "m4-1.4.7-src",
} in

{
  name = "m4-1.4.7-gcc-v5",
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
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input m4-1.4.7-src)

    WORK=/tmp/m4-1.4.7-gcc
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
    export PATH="$TOOLBIN:$SED/bin:$GREP/bin:$DIFF/bin:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL=/bin/sh
    export SHELL=/bin/sh
    export MAKEINFO=true
    unset MAKEFLAGS MFLAGS MAKELEVEL

    $BB cp -r "$SRC" "$WORK/src"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    for config_sub in $($BB find . -name config.sub -type f); do
      $BB cp "$CONFIG_SUB" "$config_sub"
      $BB chmod 755 "$config_sub"
    done
    test -f configure || { echo "ERROR: M4 configure missing" >&2; exit 1; }

    CC="$CC" \
    AR="$BINUTILS/bin/ar" \
    RANLIB="$BINUTILS/bin/ranlib" \
    ./configure \
      --prefix="$out" \
      --build="$TARGET" \
      --host="$TARGET" \
      --disable-nls \
      CPPFLAGS="-I$MUSL/include" \
      CFLAGS="-O2 -fno-common -I$MUSL/include" \
      LDFLAGS="-static"

    make -j"$JOB_COUNT" \
      CC="$CC" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      MAKEINFO=true
    make -j"$JOB_COUNT" install MAKEINFO=true

    test -x "$out/bin/m4"
    "$out/bin/m4" --version > "$WORK/version.txt"
    $BB grep -q 'GNU M4 1.4.7' "$WORK/version.txt"
    $BB cat > "$WORK/smoke.m4" <<'EOF'
define(`mantle_value', `42')mantle_value
EOF
    macro_output=$("$out/bin/m4" "$WORK/smoke.m4")
    test "$macro_output" = 42

    $BB cat > "$WORK/freeze.m4" <<'EOF'
define(`frozen_value', `frozen-ok')
EOF
    "$out/bin/m4" --freeze-state="$WORK/state.m4f" "$WORK/freeze.m4"
    test -s "$WORK/state.m4f"
    $BB printf '%s\n' 'frozen_value' > "$WORK/reload.m4"
    reload_output=$("$out/bin/m4" --reload-state="$WORK/state.m4f" "$WORK/reload.m4")
    test "$reload_output" = frozen-ok

    if "$out/bin/m4" --reload-state="$WORK/missing-state.m4f" "$WORK/reload.m4"; then
      echo "ERROR: M4 accepted a missing frozen state" >&2
      exit 1
    fi

    echo 'GNU M4 1.4.7 GCC runtime and frozen-state checks passed'
  "%],
  inputs = [base, grep, m4_src],
} | crunch.Derivation
