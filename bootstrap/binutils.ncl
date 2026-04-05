# Build GNU Binutils from source using the bootstrap toolchain.
#
# Part of the Nix-free bootstrap chain (Phase 4):
#   musl-gcc (fetched) → make → dash → binutils (this file) → ...
#
# Unlike make/dash, binutils is large enough that we use its
# autoconf configure script instead of hand-writing config.h.
# We pre-set GREP/SED/AWK to bypass configure's long-line probes
# (which busybox grep fails).
#
# Usage:
#   crunch build bootstrap/binutils.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in

let binutils_src = crunch.fetchTarball {
  url = "https://ftp.gnu.org/gnu/binutils/binutils-2.42.tar.gz",
  hash = "sha256-BS7Dn2vTeeZyfTeKjeGF+U3AcP3RChzTncadOugP1mQ=",
  name = "binutils-src",
} in

{
  name = "binutils",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink sleep touch date od \
                 uname id hostname diff cmp true false tee nl \
                 cut tar gzip gunzip bzip2 patch mktemp rmdir \
                 env seq printf dd; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      # Find bootstrap tools.
      TOOLCHAIN=""
      for d in /nix/store/*-musl-gcc; do
        if [ -d "$d/bin" ]; then TOOLCHAIN="$d"; break; fi
      done
      MAKE=""
      for d in /nix/store/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done
      DASH=""
      for d in /nix/store/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d/bin/dash"; break; fi
      done
      for tool in TOOLCHAIN MAKE DASH; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
      done

      # Symlink dash as sh (configure needs /tmp/tools/sh).
      $BB ln -sf "$DASH" /tmp/tools/sh

      # Find binutils source.
      SRCDIR=""
      for d in /nix/store/*-binutils-src; do
        if [ -d "$d" ]; then SRCDIR="$d"; break; fi
      done
      if [ -z "$SRCDIR" ]; then
        echo "ERROR: binutils-src not found" >&2; exit 1
      fi

      export PATH="/tmp/tools:$TOOLCHAIN/bin"
      export SHELL="$DASH"
      export CONFIG_SHELL="$DASH"
      # The musl.cc native toolchain has prefixed names for gcc/g++
      # but unprefixed for binutils tools (ar, ld, as, etc.).
      export CC="$TOOLCHAIN/bin/x86_64-linux-musl-gcc"
      export CXX="$TOOLCHAIN/bin/x86_64-linux-musl-g++"
      export AR="$TOOLCHAIN/bin/ar"
      export RANLIB="$TOOLCHAIN/bin/ranlib"
      export NM="$TOOLCHAIN/bin/nm"
      export STRIP="$TOOLCHAIN/bin/strip"
      export AS="$TOOLCHAIN/bin/as"
      export LD="$TOOLCHAIN/bin/ld"
      export CFLAGS="-O2 -static"
      export CXXFLAGS="-O2 -static"
      export LDFLAGS="-static"

      # Pre-set tools to bypass configure's long-line probes.
      # Busybox grep/sed fail the GNU-detection and fallback tests.
      export GREP="/tmp/tools/grep"
      export EGREP="/tmp/tools/grep -E"
      export FGREP="/tmp/tools/grep -F"
      export SED="/tmp/tools/sed"
      export AWK="/tmp/tools/awk"

      # Pre-set ar interface to skip the probe (it fails because
      # the sub-configure's eval of "$AR cru" breaks when AR has
      # an absolute /nix/store path with the sandbox shell).
      export am_cv_ar_interface=ar

      # Copy source to writable /tmp.
      mkdir -p /tmp/binutils-build
      $BB cp -r "$SRCDIR" /tmp/binutils-src 2>/dev/null
      $BB chmod -R u+w /tmp/binutils-src

      cd /tmp/binutils-build

      # Touch pre-generated files so make doesn't try to regenerate
      # them with yacc/bison/flex/lex (which we don't have).
      find /tmp/binutils-src -name '*.c' -newer /tmp/binutils-src -exec touch {} + 2>/dev/null
      for f in /tmp/binutils-src/ld/ldgram.c /tmp/binutils-src/ld/ldgram.h \
               /tmp/binutils-src/ld/ldlex.c /tmp/binutils-src/ld/deffilep.c \
               /tmp/binutils-src/ld/deffilep.h \
               /tmp/binutils-src/gas/itbl-parse.c /tmp/binutils-src/gas/itbl-parse.h \
               /tmp/binutils-src/gas/itbl-lex.c \
               /tmp/binutils-src/binutils/arparse.c /tmp/binutils-src/binutils/arparse.h \
               /tmp/binutils-src/binutils/arlex.c \
               /tmp/binutils-src/binutils/sysinfo.c /tmp/binutils-src/binutils/sysinfo.h \
               /tmp/binutils-src/binutils/syslex.c \
               /tmp/binutils-src/binutils/defparse.c /tmp/binutils-src/binutils/defparse.h \
               /tmp/binutils-src/binutils/deflex.c \
               /tmp/binutils-src/binutils/rcparse.c /tmp/binutils-src/binutils/rcparse.h \
               /tmp/binutils-src/binutils/rclex.c; do
        if [ -f "$f" ]; then
          touch "$f"
        fi
      done

      echo "=== Configuring binutils ==="
      "$DASH" /tmp/binutils-src/configure \
        --prefix=$out \
        --target=x86_64-linux-musl \
        --disable-nls \
        --disable-werror \
        --disable-gdb \
        --disable-gdbserver \
        --disable-sim \
        --disable-gold \
        --disable-gprofng \
        --disable-gprof \
        --disable-plugins \
        --disable-shared \
        --enable-static \
        --enable-deterministic-archives \
        --with-system-zlib=no \
        --with-sysroot=/ \
        2>&1 || exit 1

      echo "=== Building binutils ==="
      "$MAKE" -j4 SHELL="$DASH" MAKEINFO=true 2>&1 || exit 1

      echo "=== Installing binutils ==="
      "$MAKE" install SHELL="$DASH" MAKEINFO=true 2>&1 || exit 1

      echo "=== Verify ==="
      $out/bin/x86_64-linux-musl-as --version | head -1
      $out/bin/x86_64-linux-musl-ld --version | head -1
      $out/bin/x86_64-linux-musl-ar --version | head -1
      ls $out/bin/
    "%,
  ],
  inputs = [toolchain, gnumake, dash, binutils_src],
} | crunch.Derivation
