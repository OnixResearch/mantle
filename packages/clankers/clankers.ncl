# Build the root Clankers binary from a fixed local source+vendor bundle.
# The bundle itself is intentionally stored outside git; see
# packages/clankers/clankers-root-bundle.json for fixed hashes and the
# generation recipe.

let crunch = import "../../lib/lib.ncl" in
let seed = import "../../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_target = seed.target in
let rust = import "../../bootstrap/rust.ncl" in
let musl = import "../../bootstrap/musl.ncl" in

let src = crunch.fetchTarball {
  url = "file://.crunch-drain/artifacts/clankers-root-bundle.tar.zst",
  hash = "sha256-Kxh6OEdoNRj94tyQZyqGoOEqMFx5QJmpCwCH4V1LQ/o=",
  name = "clankers-root-bundle",
} in

{
  name = "clankers-0.1.0",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  outputs = ["out"],
  env = {
    SRC = "${src}",
    PATH = "${toolchain}/bin:${rust}/bin:${musl}/bin:/bin:/usr/bin",
    CARGO_NET_OFFLINE = "true",
    CARGO_HOME = "/tmp/cargo-home",
    CARGO_TARGET_DIR = "/tmp/cargo-target",
    RUSTUP_TOOLCHAIN = "stable",
    CC_x86_64_unknown_linux_musl = "%{seed_target}-gcc",
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "%{seed_target}-gcc",
  },
  args = [
    "-c",
    m%"
      set -eu
      export PATH="${toolchain}/bin:${rust}/bin:${musl}/bin:/bin:/usr/bin"
      export CARGO_NET_OFFLINE=true
      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      export RUSTUP_TOOLCHAIN=stable
      export CC_x86_64_unknown_linux_musl="%{seed_target}-gcc"
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="%{seed_target}-gcc"

      rm -rf /tmp/build "$CARGO_HOME" "$CARGO_TARGET_DIR"
      mkdir -p /tmp/build "$CARGO_HOME" "$CARGO_TARGET_DIR"
      cp -r "$SRC/." /tmp/build/
      chmod -R u+w /tmp/build "$CARGO_HOME" "$CARGO_TARGET_DIR"
      cd /tmp/build

      # The bundle carries Cargo source replacement for crates.io/git sources.
      # Append the musl linker target without rewriting the vendored-source map.
      mkdir -p .cargo
      cat >> .cargo/config.toml << CARGOEOF

[target.x86_64-unknown-linux-musl]
linker = "%{seed_target}-gcc"

[net]
offline = true
CARGOEOF

      cargo build --release --locked --offline --bin clankers --target x86_64-unknown-linux-musl

      mkdir -p "$out/bin" "$out/share/clankers"
      cp "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/clankers" "$out/bin/clankers"
      chmod +x "$out/bin/clankers"
      cat > "$out/share/clankers/clankers-build-receipt.txt" << RECEIPT
package=clankers
cargo_command=cargo build --release --locked --offline --bin clankers --target x86_64-unknown-linux-musl
bundle_url=file://.crunch-drain/artifacts/clankers-root-bundle.tar.zst
bundle_recursive_hash=sha256-Kxh6OEdoNRj94tyQZyqGoOEqMFx5QJmpCwCH4V1LQ/o=
artifact=$out/bin/clankers
RECEIPT
      test -x "$out/bin/clankers"
    "%,
  ],
  inputs = [toolchain, musl, rust, src],
} | crunch.Derivation
