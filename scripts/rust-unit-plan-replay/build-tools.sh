#!/usr/bin/env bash
# Run detached from the isolated worktree's pinned Nix development shell.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
HERE=$WT/scripts/rust-unit-plan-replay
TARGET=$R/targets/rust-efficiency-20261011
RUST=/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08
GLIBC_STATIC=/nix/store/j3qmic0j5df9qr54cv4p3hsd6cmprcdk-glibc-2.42-51-static
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/h75l4bwd606k94l22ajqdb98vfar5x66-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
cd "$WT"
test -x "$RUST/bin/cargo" && test -d "$GLIBC_STATIC/lib"
export CARGO_TARGET_DIR=$TARGET
cargo --config 'target.x86_64-unknown-linux-gnu.linker="cc"' build --release --locked --offline -p mantle --bin mantle
export CARGO_TARGET_DIR=$TARGET/static
export RUSTFLAGS="-C target-feature=+crt-static -L native=$GLIBC_STATIC/lib"
cargo --config 'target.x86_64-unknown-linux-gnu.linker="cc"' build --locked --offline --target x86_64-unknown-linux-gnu -p mantle-cargo-unit-plan --bin mantle-cargo-unit-helper --bin mantle-cargo-unit-producer
unset RUSTFLAGS
"$RUST/bin/rustc" --edition=2024 -C target-feature=+crt-static -L "native=$GLIBC_STATIC/lib" "$HERE/app_consumer.rs" -o "$TARGET/app-consumer"
for executable in "$TARGET/static/x86_64-unknown-linux-gnu/debug/mantle-cargo-unit-helper" "$TARGET/static/x86_64-unknown-linux-gnu/debug/mantle-cargo-unit-producer" "$TARGET/app-consumer"; do
  test -x "$executable"
  file "$executable"
  if readelf -l "$executable" | grep INTERP >/dev/null; then echo "runtime interpreter in $executable" >&2; exit 1; fi
  if readelf -d "$executable" | grep NEEDED >/dev/null; then echo "dynamic dependency in $executable" >&2; exit 1; fi
done
sha256sum "$TARGET/release/mantle" "$TARGET/static/x86_64-unknown-linux-gnu/debug/mantle-cargo-unit-helper" "$TARGET/static/x86_64-unknown-linux-gnu/debug/mantle-cargo-unit-producer" "$TARGET/app-consumer"
