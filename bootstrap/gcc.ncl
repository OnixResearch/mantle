# Build GCC from source using the bootstrap toolchain.
#
# Part of the Nix-free bootstrap chain (Phase 4):
#   seed → make → dash → binutils → musl → gcc (this file)
#
# This builds a minimal C-only GCC targeting musl libc. Combined with
# the from-source binutils and musl, this gives a fully from-source
# toolchain — the reduced bootstrap seed is no longer needed for
# subsequent builds.
#
# GCC needs three math libraries (GMP, MPFR, MPC) which are fetched
# separately and symlinked into the source tree for in-tree building.
#
# Usage:
#   crunch build bootstrap/gcc.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let binutils = (import "binutils.ncl") in
let musl = (import "musl.ncl") in

let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz",
  hash = "sha256-WWameciUJMmYPeqfDIPa3YmPZaZUuB6r6XpwlfjRXo8=",
  name = "gcc-src",
} in

let gmp_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gmp/gmp-6.2.1.tar.bz2",
  hash = "sha256-69A2yy9AY8/OFO5FfNmsvd01vTknitC/wrQSjaZq86A=",
  name = "gmp-src",
} in

let mpfr_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/mpfr/mpfr-4.1.0.tar.bz2",
  hash = "sha256-Q8xRKVAvE1c0uPSWy/Kav0Hfw0UxlnWzePXE13u6sDw=",
  name = "mpfr-src",
} in

let mpc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/mpc/mpc-1.2.1.tar.gz",
  hash = "sha256-nyt5XHt58wWilbLRlIDuECmD3qBqmKTx4jiGL/7n5Ws=",
  name = "mpc-src",
} in

{
  name = "gcc",
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
                 cut printf dd env seq tar gzip gunzip bzip2 \
                 mktemp rmdir; do
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
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done

      for tool in TOOLCHAIN MAKE DASH BINUTILS MUSL; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      $BB ln -sf "$DASH" /tmp/tools/sh

      # egrep/fgrep wrappers. Must use /tmp/tools/sh as interpreter.
      echo '#!/tmp/tools/sh' > /tmp/tools/egrep
      echo 'exec /tmp/tools/grep -E "$@"' >> /tmp/tools/egrep
      $BB chmod +x /tmp/tools/egrep
      echo '#!/tmp/tools/sh' > /tmp/tools/fgrep
      echo 'exec /tmp/tools/grep -F "$@"' >> /tmp/tools/fgrep
      $BB chmod +x /tmp/tools/fgrep

      # Find source tarballs.
      GCC_SRC="" GMP_SRC="" MPFR_SRC="" MPC_SRC=""
      for d in $NIX_STORE/*-gcc-src; do
        if [ -d "$d" ]; then GCC_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-gmp-src; do
        if [ -d "$d" ]; then GMP_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-mpfr-src; do
        if [ -d "$d" ]; then MPFR_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-mpc-src; do
        if [ -d "$d" ]; then MPC_SRC="$d"; fi
      done

      for src in GCC_SRC GMP_SRC MPFR_SRC MPC_SRC; do
        eval val=\$$src
        if [ -z "$val" ]; then
          echo "ERROR: $src not found" >&2; exit 1
        fi
      done

      # Create prefixed symlinks for the unprefixed binutils tools.
      # libtool and sub-configures look for target-prefixed binutils names.
      for tool in ar as ld ld.bfd nm objcopy objdump ranlib readelf strip size strings; do
        if [ -x "$TOOLCHAIN/bin/$tool" ]; then
          $BB ln -sf "$TOOLCHAIN/bin/$tool" /tmp/tools/%{seed_target}-$tool
        fi
      done

      export PATH="/tmp/tools:$TOOLCHAIN/bin:$BINUTILS/bin"
      export SHELL="$DASH"
      export CONFIG_SHELL="$DASH"

      # Copy gcc source to writable /tmp and symlink prerequisites.
      $BB cp -r "$GCC_SRC" /tmp/gcc-src 2>/dev/null
      $BB chmod -R u+w /tmp/gcc-src

      # Handle fetchTarball not stripping top-level dir.
      if [ -f /tmp/gcc-src/configure ]; then
        GSRC=/tmp/gcc-src
      else
        for sub in /tmp/gcc-src/*/configure; do
          GSRC=$(dirname "$sub"); break
        done
      fi

      # Symlink in-tree prerequisites (same as download_prerequisites).
      for lib in gmp mpfr mpc; do
        eval srcdir=\$$(echo $lib | tr a-z A-Z)_SRC
        # Find the actual source dir (may have version subdir)
        if [ -f "$srcdir/configure" ]; then
          ln -sf "$srcdir" "$GSRC/$lib"
        else
          for sub in $srcdir/*/configure; do
            ln -sf "$(dirname "$sub")" "$GSRC/$lib"
            break
          done
        fi
        echo "Linked $lib -> $(readlink "$GSRC/$lib")"
      done

      # Touch pre-generated flex/yacc/gperf files so make doesn't
      # try to regenerate them (we have no flex/bison/gperf).
      find $GSRC -name '*.cc' -exec touch {} + 2>/dev/null
      find $GSRC -name '*.c' -exec touch {} + 2>/dev/null
      find $GSRC -name '*.h' -exec touch {} + 2>/dev/null

      mkdir -p /tmp/gcc-build
      cd /tmp/gcc-build

      # Create a CC wrapper that always passes -static. Inside the
      # bwrap sandbox, there's no dynamic linker at /lib/ so all
      # configure test binaries and build tools must be static.
      cat > /tmp/tools/seed-cc-static << WRAPEOF
#!/bin/sh
exec $TOOLCHAIN/bin/%{seed_target}-gcc -static "\$@"
WRAPEOF
      chmod +x /tmp/tools/seed-cc-static

      cat > /tmp/tools/seed-cxx-static << WRAPEOF
#!/bin/sh
exec $TOOLCHAIN/bin/%{seed_target}-g++ -static "\$@"
WRAPEOF
      chmod +x /tmp/tools/seed-cxx-static

      echo "=== Configuring gcc ==="
      "$DASH" "$GSRC/configure" \
        --prefix=$out \
        --build=%{seed_target} \
        --host=%{seed_target} \
        --target=%{seed_target} \
        --enable-languages=c \
        --disable-bootstrap \
        --disable-multilib \
        --disable-nls \
        --disable-werror \
        --disable-libssp \
        --disable-libgomp \
        --disable-libquadmath \
        --disable-libstdcxx \
        --disable-libatomic \
        --disable-libvtv \
        --disable-libsanitizer \
        --disable-shared \
        --disable-threads \
        --disable-plugin \
        --disable-lto \
        --disable-decimal-float \
        --disable-fixed-point \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-build-sysroot="$MUSL" \
        CC="/tmp/tools/seed-cc-static" \
        CXX="/tmp/tools/seed-cxx-static" \

        AR="$TOOLCHAIN/bin/%{seed_target}-ar" \
        RANLIB="$TOOLCHAIN/bin/%{seed_target}-ranlib" \
        AS="$BINUTILS/bin/%{seed_target}-as" \
        LD="$BINUTILS/bin/%{seed_target}-ld" \
        NM="$TOOLCHAIN/bin/%{seed_target}-nm" \
        CFLAGS="-O2" \
        CXXFLAGS="-O2" \
        LDFLAGS="-static" \
        GREP="/tmp/tools/grep" \
        EGREP="/tmp/tools/grep -E" \
        FGREP="/tmp/tools/grep -F" \
        SED="/tmp/tools/sed" \
        AWK="/tmp/tools/awk" \
        am_cv_ar_interface=ar \
        2>&1 || exit 1

      echo "=== Building gcc ==="
      "$MAKE" -j4 SHELL="$DASH" MAKEINFO=true \
        2>&1 || exit 1

      echo "=== Installing gcc ==="
      "$MAKE" install SHELL="$DASH" MAKEINFO=true \
        2>&1 || exit 1

      echo "=== Verify ==="
      $out/bin/%{seed_target}-gcc --version | head -1
      ls $out/bin/
    "%,
  ],
  inputs = [toolchain, gnumake, dash, binutils, musl, gcc_src, gmp_src, mpfr_src, mpc_src],
} | crunch.Derivation
