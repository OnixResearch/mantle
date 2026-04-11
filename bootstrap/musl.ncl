# Build musl libc from source using the bootstrap toolchain.
#
# Part of the Nix-free bootstrap chain (Phase 4):
#   musl-gcc (fetched) → make → dash → binutils → musl (this file) → ...
#
# musl has a hand-written configure (not autoconf) — 843 lines,
# no long-line probes, no bison/flex. The cleanest build in the chain.
#
# Usage:
#   crunch build bootstrap/musl.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in

let musl_src = crunch.fetchTarball {
  url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz",
  hash = "sha256-WLIl4Mb82rsOQm/nQ9TmBD067Nxqm6yNwGGDZ1jS+Y8=",
  name = "musl-src",
} in

{
  name = "musl",
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
                 cut printf dd env seq; do
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

      $BB ln -sf "$DASH" /tmp/tools/sh

      # Find musl source.
      SRCDIR=""
      for d in $NIX_STORE/*-musl-src; do
        if [ -d "$d" ]; then SRCDIR="$d"; break; fi
      done
      if [ -z "$SRCDIR" ]; then
        echo "ERROR: musl-src not found" >&2; exit 1
      fi

      export PATH="/tmp/tools:$TOOLCHAIN/bin"
      export SHELL="$DASH"

      # Copy source to writable /tmp.
      $BB cp -r "$SRCDIR" /tmp/musl-src 2>/dev/null
      $BB chmod -R u+w /tmp/musl-src

      # Debug: what's in the source copy?
      echo "SRCDIR=$SRCDIR"
      ls /tmp/musl-src/ | head -10
      ls /tmp/musl-src/configure 2>/dev/null || echo "No configure at root"
      # If fetchTarball didn't strip, look one level deeper
      if [ ! -f /tmp/musl-src/configure ]; then
        for sub in /tmp/musl-src/*/configure; do
          if [ -f "$sub" ]; then
            MUSL_SRC=$(dirname "$sub")
            echo "Found configure at $MUSL_SRC"
            break
          fi
        done
      else
        MUSL_SRC=/tmp/musl-src
      fi

      mkdir -p /tmp/musl-build
      cd /tmp/musl-build

      echo "=== Configuring musl ==="
      "$DASH" $MUSL_SRC/configure \
        --prefix=$out \
        --syslibdir=$out/lib \
        --target=%{seed_target} \
        CC="$TOOLCHAIN/bin/%{seed_target}-gcc" \
        AR="$TOOLCHAIN/bin/%{seed_target}-ar" \
        RANLIB="$TOOLCHAIN/bin/%{seed_target}-ranlib" \
        CFLAGS="-O2 -pipe" \
        2>&1 || exit 1

      echo "=== Building musl ==="
      "$MAKE" -j4 AR="$TOOLCHAIN/bin/%{seed_target}-ar" RANLIB="$TOOLCHAIN/bin/%{seed_target}-ranlib" \
        2>&1 || exit 1

      echo "=== Installing musl ==="
      "$MAKE" install AR="$TOOLCHAIN/bin/%{seed_target}-ar" RANLIB="$TOOLCHAIN/bin/%{seed_target}-ranlib" \
        2>&1 || exit 1

      echo "=== Verify ==="
      ls $out/lib/libc.a
      ls $out/lib/libc.so
      ls $out/lib/%{seed_dynamic_linker}
      ls $out/include/stdio.h
      echo "musl installed: $(ls $out/lib/*.a | wc -l) static libs, $(ls $out/include/*.h | wc -l) headers"
    "%,
  ],
  inputs = [toolchain, gnumake, dash, musl_src],
} | crunch.Derivation
