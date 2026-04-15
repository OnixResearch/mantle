# Build GNU Binutils from source using the bootstrap toolchain.
#
# Part of the Nix-free bootstrap chain (Phase 4):
#   seed → make → dash → binutils (this file) → ...
#
# Unlike make/dash, binutils is large enough that we use its
# autoconf configure script instead of hand-writing config.h.
# We pre-set GREP/SED/AWK to bypass configure's long-line probes
# (which busybox grep fails).
#
# Usage:
#   crunch build bootstrap/binutils.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in

let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.42.tar.gz",
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
                 env seq printf dd uniq; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      # Find bootstrap tools.
      TOOLCHAIN=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -d "$d/bin" ]; then TOOLCHAIN="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
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

      # egrep/fgrep wrappers (sub-configures use them directly).
      # Must use /tmp/tools/sh as interpreter — /bin/sh is busybox
      # which doesn't see /tmp/tools in PATH.
      echo '#!/tmp/tools/sh' > /tmp/tools/egrep
      echo 'exec /tmp/tools/grep -E "$@"' >> /tmp/tools/egrep
      $BB chmod +x /tmp/tools/egrep
      echo '#!/tmp/tools/sh' > /tmp/tools/fgrep
      echo 'exec /tmp/tools/grep -F "$@"' >> /tmp/tools/fgrep
      $BB chmod +x /tmp/tools/fgrep

      # Stub out perl/pod2man — create empty output files.
      # pod2man is invoked as: pod2man ... > output.1
      # perl is invoked as: perl ... > output
      # Since make redirects output, just producing nothing is fine.
      # But some recipes check exit code, so exit 0.
      echo '#!/tmp/tools/sh' > /tmp/tools/pod2man
      echo 'exit 0' >> /tmp/tools/pod2man
      $BB chmod +x /tmp/tools/pod2man
      echo '#!/tmp/tools/sh' > /tmp/tools/perl
      echo 'exit 0' >> /tmp/tools/perl
      $BB chmod +x /tmp/tools/perl

      # Find binutils source.
      SRCDIR=""
      for d in $NIX_STORE/*-binutils-src; do
        if [ -d "$d" ]; then SRCDIR="$d"; break; fi
      done
      if [ -z "$SRCDIR" ]; then
        echo "ERROR: binutils-src not found" >&2; exit 1
      fi

      export PATH="/tmp/tools:$TOOLCHAIN/bin"
      export SHELL="$DASH"
      export CONFIG_SHELL="$DASH"
      # The reduced seed provider exposes prefixed gcc/g++ frontends,
      # but the raw binutils payload underneath is still unprefixed.
      export CC="$TOOLCHAIN/bin/%{seed_target}-gcc"
      export CXX="$TOOLCHAIN/bin/%{seed_target}-g++"
      export AR="$TOOLCHAIN/bin/%{seed_target}-ar"
      export RANLIB="$TOOLCHAIN/bin/%{seed_target}-ranlib"
      export NM="$TOOLCHAIN/bin/%{seed_target}-nm"
      export STRIP="$TOOLCHAIN/bin/%{seed_target}-strip"
      export AS="$TOOLCHAIN/bin/%{seed_target}-as"
      export LD="$TOOLCHAIN/bin/%{seed_target}-ld"
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

      # Touch ALL pre-generated .c/.h files so make doesn't try to
      # regenerate them with yacc/bison/flex/lex (which we don't have).
      find /tmp/binutils-src -name '*.c' -exec touch {} + 2>/dev/null
      find /tmp/binutils-src -name '*.h' -exec touch {} + 2>/dev/null

      echo "=== Configuring binutils ==="
      "$DASH" /tmp/binutils-src/configure \
        --prefix=$out \
        --target=%{seed_target} \
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
      $out/bin/%{seed_target}-as --version | head -1
      $out/bin/%{seed_target}-ld --version | head -1
      $out/bin/%{seed_target}-ar --version | head -1
      ls $out/bin/
    "%,
  ],
  inputs = [toolchain, gnumake, dash, binutils_src],
} | crunch.Derivation
