# Fetch and install a pre-built Rust toolchain (rustc + cargo + std).
#
# Uses the official standalone installer for x86_64-unknown-linux-musl.
# The resulting binaries are dynamically linked against musl, so they
# need /lib/ld-musl-x86_64.so.1 — provided by the from-source musl.
#
# Part of the self-build chain:
#   musl-gcc (fetched) -> make -> dash -> binutils -> musl -> gcc
#   rust (this file, fetched pre-built)
#   -> crunch (self-build)
#
# Usage:
#   crunch build bootstrap/rust.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

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
      for d in /nix/store/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      RUST_SRC=""
      for d in /nix/store/*-rust-standalone; do
        if [ -d "$d/rustc" ]; then RUST_SRC="$d"; break; fi
      done
      DASH=""
      for d in /nix/store/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done

      for tool in MUSL RUST_SRC DASH; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      # The Rust binaries need /lib/ld-musl-x86_64.so.1.
      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/ld-musl-x86_64.so.1 2>/dev/null || true

      # Also need libgcc_s.so.1 for unwinding.
      MUSL_GCC=""
      for d in /nix/store/*-musl-gcc; do
        if [ -f "$d/lib/libgcc_s.so.1" ]; then MUSL_GCC="$d"; break; fi
      done
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$MUSL_GCC/lib"
      fi

      # The install.sh script needs a real shell (not busybox sh).
      $BB ln -sf "$DASH/bin/dash" /tmp/tools/sh

      echo "=== Installing Rust toolchain ==="

      # Skip install.sh (needs bash). Copy components directly.
      mkdir -p $out/bin $out/lib

      # rustc
      cp $RUST_SRC/rustc/bin/rustc $out/bin/
      cp $RUST_SRC/rustc/bin/rustdoc $out/bin/
      cp -r $RUST_SRC/rustc/lib/* $out/lib/

      # cargo
      cp $RUST_SRC/cargo/bin/cargo $out/bin/

      # rust-std (standard library for the musl target)
      cp -r $RUST_SRC/rust-std-x86_64-unknown-linux-musl/lib/rustlib $out/lib/rustlib-std-tmp
      # Merge into the rustc rustlib
      if [ -d $out/lib/rustlib/x86_64-unknown-linux-musl ]; then
        # Already have the target dir from rustc, just add std libs
        cp $RUST_SRC/rust-std-x86_64-unknown-linux-musl/lib/rustlib/x86_64-unknown-linux-musl/lib/* \
          $out/lib/rustlib/x86_64-unknown-linux-musl/lib/
      else
        cp -r $RUST_SRC/rust-std-x86_64-unknown-linux-musl/lib/rustlib/x86_64-unknown-linux-musl \
          $out/lib/rustlib/
      fi
      rm -rf $out/lib/rustlib-std-tmp

      echo "=== Verify ==="
      ls -la /lib/ld-musl-x86_64.so.1
      "$out/bin/rustc" --version
      "$out/bin/cargo" --version

      echo "=== Installed files ==="
      ls "$out/bin/"
    "%,
  ],
  inputs = [toolchain, gnumake, dash, musl, rust_src],
} | crunch.Derivation
