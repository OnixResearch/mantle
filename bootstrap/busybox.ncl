# Build busybox-static from source using the bootstrap GCC + musl.
#
# Part of the Nix-free bootstrap chain:
#   musl-gcc (fetched) -> make -> dash -> binutils -> musl -> gcc
#     -> busybox (this file)
#
# Produces a single statically-linked busybox binary with all default
# applets. Used as the sandbox shell for crunch builds, replacing the
# external busybox-static dependency.
#
# Usage:
#   crunch build bootstrap/busybox.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let musl = (import "musl.ncl") in
let gcc = (import "gcc.ncl") in

let busybox_src = crunch.fetchTarball {
  url = "https://busybox.net/downloads/busybox-1.37.0.tar.bz2",
  hash = "sha256-tDFhcU0GUX3/ct2Hw+Ad8iNyMl0GWovhQynLgooh+qU=",
  name = "busybox-src",
} in

{
  name = "busybox",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -e
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink sleep touch date od \
                 uname id hostname diff cmp true false tee sh; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      # Find GCC.
      GCC=""
      for d in /nix/store/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      if [ -z "$GCC" ]; then
        echo "ERROR: gcc not found" >&2; exit 1
      fi
      export PATH="$GCC/bin:$PATH"

      # Find GNU Make.
      MAKE=""
      for d in /nix/store/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done
      if [ -z "$MAKE" ]; then
        echo "ERROR: gnumake not found" >&2; exit 1
      fi

      # Find dash for /bin/sh replacement.
      DASH=""
      for d in /nix/store/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d/bin/dash"; break; fi
      done

      # Find busybox source.
      SRCDIR=""
      for d in /nix/store/*-busybox-src; do
        if [ -d "$d" ]; then SRCDIR="$d"; break; fi
      done
      if [ -z "$SRCDIR" ]; then
        echo "ERROR: busybox-src not found" >&2; exit 1
      fi

      # Copy source to writable location (sandbox source mounts are read-only).
      cp -r "$SRCDIR" /tmp/build 2>/dev/null
      chmod -R u+w /tmp/build
      cd /tmp/build

      # Set up musl library paths for GCC.
      # The bootstrap GCC targets musl, so -static links against musl libc.
      GCC_LIB=$(dirname $(gcc -print-libgcc-file-name))
      export LIBRARY_PATH="$GCC_LIB:${LIBRARY_PATH:-}"

      # Create musl dynamic linker symlink for host tools.
      # The bootstrap GCC produces musl-linked binaries. Host tools
      # (fixdep etc.) run during the build and need the dynamic linker.
      MUSL=""
      for d in /nix/store/*-musl; do
        if [ -f "$d/lib/libc.so" ]; then MUSL="$d"; break; fi
      done
      if [ -z "$MUSL" ]; then
        # Fall back to the seed musl-gcc toolchain.
        for d in /nix/store/*-musl-gcc; do
          if [ -f "$d/lib/libc.so" ]; then MUSL="$d"; break; fi
        done
      fi
      if [ -n "$MUSL" ]; then
        $BB mkdir -p /lib 2>/dev/null || true
        $BB ln -sf "$MUSL/lib/libc.so" /lib/ld-musl-x86_64.so.1 2>/dev/null || true
      fi

      # Compute job count — NIX_BUILD_CORES is 0 in sandbox, which is
      # invalid for make. Default to 4 if 0 or unset.
      JOBS=${NIX_BUILD_CORES:-4}
      if [ "$JOBS" = "0" ] || [ -z "$JOBS" ]; then JOBS=4; fi

      # Host tools (scripts/basic/fixdep) must be statically linked
      # because the bwrap sandbox has no dynamic linker at /lib/ld-musl-*.
      # Set HOSTCC to include -static so kbuild's single-step compile+link
      # produces a static binary.
      export HOSTCC="gcc -static"

      # Generate default config.
      $MAKE defconfig CC=gcc

      # Patch config for static build.
      sed -i 's/# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
      sed -i 's/# CONFIG_INSTALL_NO_USR is not set/CONFIG_INSTALL_NO_USR=y/' .config

      echo "Building busybox with static musl linking..."
      $MAKE -j$JOBS \
        CC=gcc \
        HOSTCC=gcc \
        HOSTLDFLAGS="-static" \
        LDFLAGS="-static" \
        CFLAGS="-Os -pipe" \
        V=0 || { echo "ERROR: busybox build failed" >&2; exit 1; }

      # Verify the binary was produced.
      if [ ! -f busybox ]; then
        echo "ERROR: busybox binary not found after build" >&2
        exit 1
      fi

      # Install.
      mkdir -p $out/bin
      cp busybox $out/bin/busybox
      chmod +x $out/bin/busybox

      # Create symlinks for all applets.
      for applet in $($out/bin/busybox --list); do
        ln -sf busybox "$out/bin/$applet"
      done

      # Verify.
      echo "Verifying busybox..."
      $out/bin/busybox --list | head -5
      $out/bin/busybox sh -c 'echo busybox-ok'
      echo "Busybox build complete: $out/bin/busybox"
    "%
  ],
  system = "x86_64-linux",
  inputs = [toolchain, gnumake, dash, musl, gcc, busybox_src],
}
