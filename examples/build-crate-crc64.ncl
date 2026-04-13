# Build a real Rust crate from crates.io with crunch.
#
# Usage:
#   mkdir -p /tmp/crunch-examples-store /tmp/crunch-examples-state
#   crunch build examples/build-crate-crc64.ncl \
#     --store /tmp/crunch-examples-store \
#     --state-dir /tmp/crunch-examples-state \
#     --no-substitute
#
# What this does:
#   - fetches the published `crc64` crate source from crates.io
#   - builds crunch's bootstrap Rust toolchain
#   - compiles the crate with `cargo build --release --offline`
#
# `crc64` is a good example crate because version 2.0.0 has no Cargo
# dependencies, so the build stays offline inside the sandbox.

let crunch = import "lib.ncl" in
let seed = import "../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let rust = import "../bootstrap/rust.ncl" in
let musl = import "../bootstrap/musl.ncl" in

let src = crunch.fetchTarball {
  url = "https://static.crates.io/crates/crc64/crc64-2.0.0.crate",
  hash = "sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA=",
  name = "crc64-src",
} in

{
  name = "crc64-2.0.0",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools /tmp/home /tmp/build /tmp/cargo-home /tmp/cargo-target
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk tr head tail \
                 sort wc expr test basename dirname install find xargs readlink \
                 touch true false tee; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"
      export HOME=/tmp/home
      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target

      TOOLCHAIN=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -x "$d/bin/%{seed_target}-gcc" ]; then TOOLCHAIN="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -f "$d/lib/libc.so" ]; then MUSL="$d"; break; fi
      done
      RUST=""
      for d in $NIX_STORE/*-rust; do
        if [ -x "$d/bin/cargo" ]; then RUST="$d"; break; fi
      done
      SRC=""
      for d in $NIX_STORE/*-crc64-src; do
        if [ -f "$d/Cargo.toml" ]; then SRC="$d"; break; fi
      done

      for tool in TOOLCHAIN MUSL RUST SRC; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2
          exit 1
        fi
        echo "$tool=$val"
      done

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf "$MUSL/lib/libc.so" /lib/%{seed_dynamic_linker} 2>/dev/null || true

      export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"
      export LD_LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export RUSTFLAGS="-C link-arg=-L$TOOLCHAIN/%{seed_target}/lib"

      cp -r "$SRC/." /tmp/build/
      chmod -R u+w /tmp/build
      cd /tmp/build

      mkdir -p .cargo
      cat > .cargo/config.toml << CARGOEOF
[target.x86_64-unknown-linux-musl]
linker = "%{seed_target}-gcc"

[net]
offline = true
CARGOEOF

      cargo build --release --offline --bin crc64 --target x86_64-unknown-linux-musl

      mkdir -p $out/bin
      cp $CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/crc64 $out/bin/
      echo 'crunch example payload' > /tmp/crc64-input.txt
      $out/bin/crc64 /tmp/crc64-input.txt > /tmp/crc64-output.txt
      cat /tmp/crc64-output.txt
    "%,
  ],
  inputs = [toolchain, musl, rust, src],
} | crunch.Derivation
