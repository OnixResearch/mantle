# Conventional GNU Awk 3.0.4 rebuild with the pass4 GCC/musl/binutils cycle.
#
# The TinyCC-era bridge has incomplete string-length and hexadecimal-format
# behavior, which corrupts generated Perl opcode metadata. This stage validates
# those semantics directly before serving any source-regeneration consumer.

fun base grep =>
let crunch = import "lib.ncl" in
let gawk_src = import "gawk-3.0.4-src.ncl" in

{
  name = "gawk-3.0.4-gcc-v4",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    GCC_VERSION=4.0.4
    JOB_COUNT=1
    EXPECTED_STRING_LENGTH=2
    HEX_INPUT_DECIMAL=68
    EXPECTED_HEX=00000044
    SUM_LEFT=19
    SUM_RIGHT=23
    EXPECTED_SUM=42
    HEX_FORMAT=$($BB printf '\04508x')

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
    CONFIG_SUB="$BASE/share/automake/config.sub"
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input gawk-3.0.4-src)

    WORK=/tmp/gawk-3.0.4-gcc
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
    export PATH="$TOOLBIN:$SED/bin:$GREP/bin:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
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
    if [ -f config/config.sub ]; then
      $BB cp "$CONFIG_SUB" config/config.sub
      $BB chmod 755 config/config.sub
    fi
    test -f configure || { echo "ERROR: Gawk configure missing" >&2; exit 1; }

    CC="$CC" \
    AR="$BINUTILS/bin/ar" \
    RANLIB="$BINUTILS/bin/ranlib" \
    CPPFLAGS="-I$MUSL/include" \
    CFLAGS="-O2 -fno-common -I$MUSL/include" \
    LDFLAGS="-static" \
    ac_cv_type_size_t=yes \
    ac_cv_type_ssize_t=yes \
    ./configure \
      --prefix="$out" \
      --build="$TARGET" \
      --host="$TARGET" \
      --disable-nls

    make -j"$JOB_COUNT" \
      CC="$CC" \
      AR="$BINUTILS/bin/ar" \
      RANLIB="$BINUTILS/bin/ranlib" \
      MAKEINFO=true
    make -j"$JOB_COUNT" install MAKEINFO=true

    test -x "$out/bin/gawk"
    test -x "$out/bin/awk"
    "$out/bin/gawk" --version > "$WORK/version.txt"
    $BB grep -q 'GNU Awk 3.0.4' "$WORK/version.txt"

    length_value=$("$out/bin/gawk" 'BEGIN { print length("S?") }')
    test "$length_value" = "$EXPECTED_STRING_LENGTH"
    hex_value=$("$out/bin/gawk" -v value="$HEX_INPUT_DECIMAL" -v format="$HEX_FORMAT" 'BEGIN { printf format "\n", value }')
    test "$hex_value" = "$EXPECTED_HEX"
    "$out/bin/gawk" -v expected="$EXPECTED_STRING_LENGTH" 'BEGIN { count = split("S?", chars, ""); if (count != expected || chars[1] != "S" || chars[2] != "?") exit 1 }'
    sum_value=$($BB printf '%s\n%s\n' "$SUM_LEFT" "$SUM_RIGHT" | "$out/bin/gawk" '{ sum += $1 } END { print sum }')
    test "$sum_value" = "$EXPECTED_SUM"

    $BB rm -f "$WORK/rejected.txt"
    if "$out/bin/gawk" 'BEGIN {' > "$WORK/rejected.txt" 2>&1; then
      echo "ERROR: Gawk accepted a malformed program" >&2
      exit 1
    fi

    echo 'GNU Awk 3.0.4 GCC length, split, hex, numeric, and rejection checks passed'
  "%],
  inputs = [base, grep, gawk_src],
} | crunch.Derivation
