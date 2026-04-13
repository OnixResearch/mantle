# Fetch and install a pre-built Rust toolchain (rustc + cargo + std).
#
# Uses the official standalone installer for x86_64-unknown-linux-musl.
# The resulting binaries are dynamically linked against musl, so they
# need the musl dynamic linker from the bootstrap seed target.
#
# Part of the self-build chain:
#   seed -> make -> dash -> binutils -> musl -> gcc
#   rust (this file, fetched pre-built)
#   -> crunch (self-build)
#
# Usage:
#   crunch build bootstrap/rust.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let musl = (import "musl.ncl") in

let rust_src = crunch.fetchTarball {
  url = "https://static.rust-lang.org/dist/rust-1.94.1-x86_64-unknown-linux-musl.tar.xz",
  hash = "blake3-EAKuRdQqLZ75fR9ETqHkGZuxiWnBni8CpCouTQNYNvI=",
  name = "rust-standalone",
} in

{
  name = "rust",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink touch true false tee \
                 du cut uname id whoami; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      # Find inputs.
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      RUST_SRC=""
      for d in $NIX_STORE/*-rust-standalone; do
        if [ -d "$d/rustc" ]; then RUST_SRC="$d"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done

      for tool in MUSL RUST_SRC DASH; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      # The Rust binaries need the musl dynamic linker.
      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/%{seed_dynamic_linker} 2>/dev/null || true

      # Also need libgcc_s.so.1 for unwinding.
      MUSL_GCC=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -f "$d/%{seed_target}/lib/libgcc_s.so.1" ]; then MUSL_GCC="$d/%{seed_target}/lib"; break; fi
      done
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$MUSL_GCC"
        export LIBRARY_PATH="$MUSL_GCC"
      fi

      # The install.sh script needs a real shell (not busybox sh).
      $BB ln -sf "$DASH/bin/dash" /tmp/tools/sh

      set -ex
      echo "=== Installing Rust toolchain ==="

      # Skip install.sh (needs bash). Copy components directly.
      mkdir -p $out/bin $out/lib

      # rustc
      cp $RUST_SRC/rustc/bin/rustc $out/bin/
      cp $RUST_SRC/rustc/bin/rustdoc $out/bin/
      cp -r $RUST_SRC/rustc/lib/* $out/lib/
      # Fetched tarball has read-only permissions. Make writable so
      # we can add the std libs into the same rustlib tree.
      chmod -R u+w $out/lib/

      # cargo
      cp $RUST_SRC/cargo/bin/cargo $out/bin/

      # rust-std (standard library for the musl target)
      cp -r $RUST_SRC/rust-std-x86_64-unknown-linux-musl/lib/rustlib/x86_64-unknown-linux-musl/lib \
        $out/lib/rustlib/x86_64-unknown-linux-musl/

      # Keep libgcc_s next to librustc_driver so later sandboxes do not rely
      # on external loader search paths for Rust unwinding support.
      if [ -n "$MUSL_GCC" ]; then
        for lib in libgcc_s.so libgcc_s.so.1; do
          if [ -f "$MUSL_GCC/$lib" ]; then
            cp "$MUSL_GCC/$lib" $out/lib/
          fi
        done
      fi

      echo "std lib count: $(ls $out/lib/rustlib/x86_64-unknown-linux-musl/lib/ | wc -l)"

      echo "=== Verify ==="
      ls -la /lib/%{seed_dynamic_linker}
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$out/lib:$MUSL_GCC"
      else
        export LD_LIBRARY_PATH="$out/lib"
      fi
      "$out/bin/rustc" --version
      "$out/bin/cargo" --version

      echo "=== Installed files ==="
      ls "$out/bin/"
    "%,
  ],
  inputs = [toolchain, gnumake, dash, musl, rust_src],
} | crunch.Derivation
