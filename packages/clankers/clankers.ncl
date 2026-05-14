# Build the root Clankers binary from a fixed local source+vendor bundle.
# The bundle itself is intentionally stored outside git; see
# packages/clankers/clankers-root-bundle.json for fixed hashes and the
# generation recipe.

let crunch = import "../../lib/lib.ncl" in
let seed = import "../../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let rust = import "../../bootstrap/rust.ncl" in
let musl = import "../../bootstrap/musl.ncl" in

let src = crunch.fetchTarball {
  url = "file://.crunch-drain/artifacts/clankers-root-bundle.tar.zst",
  hash = "sha256-a6vnNF5mE/E3C7KlyQ9JXM77fdlQ+Yzc7sSlRi9JkAk=",
  name = "clankers-root-bundle",
} in
let native_libs = crunch.fetchTarball {
  url = "file://.crunch-drain/artifacts/clankers-native-libs-v4.tar.zst",
  hash = "sha256-jZIbj//YNZfW3W6oIoM0Q4YXEa1wS+IxV9IHmcs3oTQ=",
  name = "clankers-native-libs",
} in

{
  name = "clankers-0.1.0",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  outputs = ["out"],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools /tmp/home /tmp/build /tmp/cargo-home /tmp/cargo-target
      for cmd in cat mkdir cp chmod ln ls echo rm mv grep sed tr sort wc expr test basename dirname install find xargs readlink touch true false tee; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"
      export HOME=/tmp/home
      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      export CARGO_NET_OFFLINE=true
      export CARGO_PROFILE_RELEASE_LTO=false
      export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=32
      export CARGO_PROFILE_RELEASE_DEBUG=0
      export RUSTUP_TOOLCHAIN=stable

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
      for d in $NIX_STORE/*-clankers-root-bundle; do
        if [ -f "$d/Cargo.toml" ]; then SRC="$d"; break; fi
      done
      NATIVE_LIBS=""
      for d in $NIX_STORE/*; do
        if [ -d "$d/openssl/include/openssl" ] && [ -d "$d/sqlite-src" ]; then NATIVE_LIBS="$d"; break; fi
      done
      for tool in TOOLCHAIN MUSL RUST SRC NATIVE_LIBS; do
        eval val=\$$tool
        if [ -z "$val" ]; then echo "ERROR: $tool not found" >&2; exit 1; fi
        echo "$tool=$val"
      done

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf "$MUSL/lib/libc.so" /lib/%{seed_dynamic_linker} 2>/dev/null || true

      export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"
      export LD_LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export RUSTFLAGS="-C link-arg=-L$TOOLCHAIN/%{seed_target}/lib -C link-arg=-L/tmp/native-sqlite/lib"
      export CC_x86_64_unknown_linux_musl="%{seed_target}-gcc"
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="%{seed_target}-gcc"
      export OPENSSL_INCLUDE_DIR="$NATIVE_LIBS/openssl/include"
      export OPENSSL_LIB_DIR="$NATIVE_LIBS/openssl/lib"
      export OPENSSL_STATIC=1
      export SQLITE3_STATIC=1
      mkdir -p /tmp/native-sqlite/lib /tmp/native-sqlite/include
      cp "$NATIVE_LIBS/sqlite-src/sqlite3.h" /tmp/native-sqlite/include/sqlite3.h
      %{seed_target}-gcc -O2 -DSQLITE_THREADSAFE=1 -DSQLITE_ENABLE_FTS5 -DSQLITE_ENABLE_JSON1 -c "$NATIVE_LIBS/sqlite-src/sqlite3.c" -o /tmp/native-sqlite/sqlite3.o
      %{seed_target}-ar rcs /tmp/native-sqlite/lib/libsqlite3.a /tmp/native-sqlite/sqlite3.o
      export SQLITE3_INCLUDE_DIR="/tmp/native-sqlite/include"
      export SQLITE3_LIB_DIR="/tmp/native-sqlite/lib"

      cp -r "$SRC/." /tmp/build/
      chmod -R u+w /tmp/build "$CARGO_HOME" "$CARGO_TARGET_DIR"
      cd /tmp/build

      # The bundle carries Cargo source replacement for crates.io/git sources.
      # Append the musl linker target without rewriting the vendored-source map.
      mkdir -p .cargo
      # The root binary's `--help`/`--version` smoke does not need local ONNX TTS.
      # Clankers currently depends on clankers-tts with default features, which pulls
      # kittentts -> ort-sys -> libonnxruntime. Keep the build offline by disabling
      # that default feature in the copied bundle only.
      $BB sed -i 's|clankers-tts = { version = "0.1.0", path = "crates/clankers-tts" }|clankers-tts = { version = "0.1.0", path = "crates/clankers-tts", default-features = false }|' /tmp/build/Cargo.toml
      $BB sed -i 's|default = \["kitten"\]|default = []|' /tmp/build/crates/clankers-tts/Cargo.toml

      cat >> /tmp/build/.cargo/config.toml << CARGOEOF

[target.x86_64-unknown-linux-musl]
linker = "%{seed_target}-gcc"

[net]
offline = true
CARGOEOF

      cargo build --release --locked --offline --no-default-features --bin clankers --target x86_64-unknown-linux-musl

      mkdir -p "$out/bin" "$out/share/clankers"
      cp "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/clankers" "$out/bin/clankers"
      chmod +x "$out/bin/clankers"
      cat > "$out/share/clankers/clankers-build-receipt.txt" << RECEIPT
package=clankers
cargo_command=cargo build --release --locked --offline --no-default-features --bin clankers --target x86_64-unknown-linux-musl
bundle_url=file://.crunch-drain/artifacts/clankers-root-bundle.tar.zst
bundle_recursive_hash=sha256-a6vnNF5mE/E3C7KlyQ9JXM77fdlQ+Yzc7sSlRi9JkAk=
bundle_archive_blake3=dae16bc6eafc5b5f82aad2d709fd5e27507c2928b79754ae214e6c3c385ec799
final_proof_hash_blake3=264b66f3389075f84373967f5e79738ac4fbbf91d7ddd530d3bf0f412fb276e1
artifact=$out/bin/clankers
RECEIPT
      test -x "$out/bin/clankers"
    "%,
  ],
  inputs = [toolchain, musl, rust, src, native_libs],
} | crunch.Derivation
