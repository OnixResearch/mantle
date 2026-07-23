# Exact source-built Autoconf 2.13 for GCC 4.0 top-level regeneration.

let crunch = import "lib.ncl" in
let autoconf_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/autoconf/autoconf-2.13.tar.gz",
  hash = "sha256-Gl7VQbtxjjyFy8rCJelqLhdl4TbO3L0FPumoBQb2+28=",
  name = "autoconf-2.13-src",
} in

{
  name = "autoconf-2.13-gcc-v1",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/autoconf-2.13-gcc
      TOOLBIN="$WORK/tools"

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      SRC=$(find_input autoconf-2.13-src)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GAWK=$(find_input gawk-3.0.4-gcc-v4)
      M4_ROOT=$(find_input m4-1.4.7-gcc-v5)
      PERL_ROOT=$(find_input perl-5.6.2-configured-gcc-v16)

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN" "$WORK/build"
      for applet in basename cat chmod cmp cp cut dirname echo env expr false find grep head install ln ls mkdir mv printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$MAKE/bin/make" "$TOOLBIN/make"
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/awk"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/gawk"
      $BB ln -sf "$M4_ROOT/bin/m4" "$TOOLBIN/m4"
      $BB ln -sf "$PERL_ROOT/bin/perl" "$TOOLBIN/perl"
      export PATH="$TOOLBIN:$STAGE0/bin"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export M4="$M4_ROOT/bin/m4"
      export AWK="$GAWK/bin/awk"
      export PERL="$PERL_ROOT/bin/perl"
      export MAKEFLAGS=

      cd "$WORK/build"
      M4="$M4" \
      AWK="$GAWK/bin/awk" \
      PERL="$PERL" \
      "$SRC/configure" \
        --prefix="$out" \
        --datadir="$out/share"
      make -j1
      make -j1 install

      test -x "$out/bin/autoconf"
      test -s "$out/share/autoconf/acgeneral.m4"
      "$out/bin/autoconf" --version | "$GREP/bin/grep" -q '2.13'

      POSITIVE="$WORK/positive"
      $BB mkdir -p "$POSITIVE"
      $BB cat > "$POSITIVE/configure.in" <<'EOF'
AC_INIT(mantle-positive.txt)
AC_MSG_CHECKING(mantle-autoconf-2.13-runtime)
AC_MSG_RESULT(ok)
AC_OUTPUT
EOF
      $BB printf '%s\n' 'mantle-autoconf-2.13-positive' > "$POSITIVE/mantle-positive.txt"
      cd "$POSITIVE"
      AC_MACRODIR="$out/share/autoconf" M4="$M4" AWK="$GAWK/bin/awk" \
        "$out/bin/autoconf" configure.in > configure.first
      test -s configure.first
      AC_MACRODIR="$out/share/autoconf" M4="$M4" AWK="$GAWK/bin/awk" \
        "$out/bin/autoconf" configure.in > configure.second
      cmp configure.first configure.second
      $BB cp configure.first configure
      $BB chmod 755 configure
      ./configure --host=x86_64-unknown-linux-musl --build=x86_64-unknown-linux-musl --prefix="$POSITIVE/install" \
        | "$GREP/bin/grep" -q 'mantle-autoconf-2.13-runtime.*ok'

      NEGATIVE="$WORK/negative"
      $BB mkdir -p "$NEGATIVE"
      $BB cat > "$NEGATIVE/configure.in" <<'EOF'
AC_INIT(mantle-negative.txt)
AC_MSG_ERROR(mantle-negative)
AC_OUTPUT
EOF
      $BB printf '%s\n' 'mantle-autoconf-2.13-negative' > "$NEGATIVE/mantle-negative.txt"
      cd "$NEGATIVE"
      AC_MACRODIR="$out/share/autoconf" M4="$M4" AWK="$GAWK/bin/awk" \
        "$out/bin/autoconf" configure.in > configure
      test -s configure
      $BB chmod 755 configure
      if ./configure >negative.stdout 2>negative.stderr; then
        echo "ERROR: Autoconf 2.13 negative configure unexpectedly succeeded" >&2
        exit 1
      fi
      "$GREP/bin/grep" -q 'mantle-negative' negative.stderr

      RELOCATED="$WORK/relocated-autoconf"
      $BB cp -R "$out" "$RELOCATED"
      cd "$POSITIVE"
      AC_MACRODIR="$RELOCATED/share/autoconf" M4="$M4" AWK="$GAWK/bin/awk" \
        "$RELOCATED/bin/autoconf" configure.in > configure.relocated
      cmp configure.first configure.relocated

      $BB mkdir -p "$out/share/autoconf213-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: exact Autoconf 2.13 has component-level generation, rejection, and relocation evidence; normalized-provider admission remains separate' \
        > "$out/share/autoconf213-frontier/NON_ADMISSION"
      echo "Autoconf 2.13 deterministic generation, rejection, and copied-tree checks passed; not provider admission"
    "%
  ],
  inputs = [
    autoconf_src,
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
  ],
} | crunch.Derivation
