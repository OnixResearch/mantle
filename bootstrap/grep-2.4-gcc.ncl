# Conventional GNU grep 2.4 rebuild with the closed GCC/musl/binutils cycle.
#
# The earlier grep-2.4-musl output is an intentionally bounded shell bridge and
# does not implement count semantics required by generated Autoconf scripts.

let crunch = import "lib.ncl" in
let base = crunch.derivationFile "gcc-generator-base.ncl" in
let grep_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/grep/grep-2.4.tar.gz",
  hash = "sha256-v9xxK5dLi3FxvpCbRjPorpK+bPNcAXwj/jgX4eslWxI=",
  name = "grep-2.4-src",
} in

{
  name = "grep-2.4-gcc-v10",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    TARGET=x86_64-unknown-linux-musl
    GCC_VERSION=4.0.4
    JOB_COUNT=1
    LONG_LINE_BYTES=20000
    EXPECTED_ALPHA_MATCHES=2

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
    SRC=$(find_input grep-2.4-src)

    WORK=/tmp/grep-2.4-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN"
    for applet in awk basename cat chmod cmp cp cut dirname echo expr false find grep head ln ls mkdir mv od printf pwd rm rmdir sh sleep sort tail test touch tr true uname uniq wc; do
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
    export PATH="$TOOLBIN:$SED/bin:$BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
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
    test -f configure || { echo "ERROR: grep configure missing" >&2; exit 1; }

    CC="$CC" \
    AR="$BINUTILS/bin/ar" \
    RANLIB="$BINUTILS/bin/ranlib" \
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

    test -x "$out/bin/grep"
    "$out/bin/grep" --version > "$WORK/version.txt"
    $BB grep -q 'GNU grep' "$WORK/version.txt"
    $BB grep -q '2.4' "$WORK/version.txt"

    $BB printf 'alpha\nbeta\nalpha\n' > "$WORK/lines.txt"
    count=$("$out/bin/grep" -c '^alpha$' "$WORK/lines.txt")
    test "$count" = "$EXPECTED_ALPHA_MATCHES"
    "$out/bin/grep" -E -q '^(alpha|beta)$' "$WORK/lines.txt"
    if "$out/bin/grep" -q '^missing$' "$WORK/lines.txt"; then
      echo "ERROR: grep accepted a no-match query" >&2
      exit 1
    fi

    $BB awk -v bound="$LONG_LINE_BYTES" 'BEGIN { for (i = 0; i < bound; i++) printf "x"; print "mantle-long-line" }' > "$WORK/long-line.txt"
    "$out/bin/grep" -q 'mantle-long-line$' "$WORK/long-line.txt"

    if "$out/bin/grep" '[' "$WORK/lines.txt" > "$WORK/rejected.txt" 2>&1; then
      echo "ERROR: grep accepted a malformed regular expression" >&2
      exit 1
    fi

    echo 'GNU grep 2.4 GCC runtime, count, regex, and long-line checks passed'
  "%],
  # The normalized base is the one direct compiler/runtime/tool input.
  inputs = [base, grep_src],
} | crunch.Derivation
