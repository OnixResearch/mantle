# crunch building itself.
#
# Self-hosting: crunch compiled by crunch, using only the bootstrap
# toolchain (from-source C compiler) and a fetched Rust toolchain.
#
# Build dependencies:
#   - Rust (fetched, musl-linked) — rustc, cargo
#   - GCC (from source) — for ring's asm and cc-crate deps
#   - Binutils (from source) — ar, ld
#   - Musl (from source) — libc + dynamic linker for Rust binaries
#   - Dash (from source) — build shell
#   - Make (from source) — for C build scripts
#   - No protoc needed (pre-generated proto files)
#   - No clang/mold needed (gcc+ld override in .cargo/config.toml)
#
# The source tarball includes vendored Cargo deps (cargo vendor),
# so no network access is needed during the build.
#
# Usage:
#   crunch build bootstrap/crunch.ncl -I lib --store /tmp/crunch-store
#
# Generating the source tarball:
#   cd crunch && cargo vendor --locked vendor-deps
#   git archive --prefix=crunch-src/ HEAD -o /tmp/crunch-src.tar
#   tar rf /tmp/crunch-src.tar --transform='s,^vendor-deps/,crunch-src/vendor-deps/,' vendor-deps/
#   xz -T0 /tmp/crunch-src.tar

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let binutils = (import "binutils.ncl") in
let musl = (import "musl.ncl") in
let gcc = (import "gcc.ncl") in
let rust = (import "rust.ncl") in

let crunch_src = crunch.fetchTarball {
  url = "file:///tmp/crunch-src.tar.xz",
  hash = "blake3-+Fj/I3X+w8OdwuUACJTD/y6F91Fj7KRERZh+l4kJN24=",
  name = "crunch-src",
} in

{
  name = "crunch",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink touch true false tee \
                 du cut uname id whoami env printenv; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      # Find from-source tools.
      GCC=""
      for d in $NIX_STORE/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d"; break; fi
      done
      RUST=""
      for d in $NIX_STORE/*-rust; do
        if [ -x "$d/bin/rustc" ]; then RUST="$d"; break; fi
      done
      CRUNCH_SRC=""
      for d in $NIX_STORE/*-crunch-src; do
        if [ -f "$d/Cargo.toml" ]; then CRUNCH_SRC="$d"; break; fi
      done

      for tool in GCC BINUTILS MUSL DASH MAKE RUST CRUNCH_SRC; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      # Dynamic linker for musl-linked binaries (Rust, binutils).
      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/ld-musl-x86_64.so.1 2>/dev/null || true

      # Rust binaries need libgcc_s.so.1 for unwinding.
      # The fetched musl-gcc toolchain provides it.
      MUSL_GCC=""
      for d in $NIX_STORE/*-musl-gcc; do
        if [ -f "$d/lib/libgcc_s.so.1" ]; then MUSL_GCC="$d"; break; fi
      done
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$MUSL_GCC/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
      else
        # Fallback: from-source GCC
        if [ -f "$GCC/lib/libgcc_s.so.1" ]; then
          export LD_LIBRARY_PATH="$GCC/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
        fi
      fi

      # Shell symlink for cargo build scripts.
      $BB ln -sf "$DASH/bin/dash" /tmp/tools/sh

      # Unprefixed binutils for gcc's collect2.
      for tool in as ld ld.bfd ar nm objcopy objdump ranlib readelf strip; do
        if [ -x "$BINUTILS/bin/x86_64-linux-musl-$tool" ] && [ ! -e "/tmp/tools/$tool" ]; then
          $BB ln -sf "$BINUTILS/bin/x86_64-linux-musl-$tool" /tmp/tools/$tool
        fi
      done

      export PATH="/tmp/tools:$RUST/bin:$GCC/bin:$BINUTILS/bin:$MAKE/bin"

      echo "=== Tool versions ==="
      rustc --version
      cargo --version
      gcc --version | head -1
      make --version | head -1

      # Copy source tree (sandbox paths are read-only).
      $BB mkdir -p /tmp/build
      cp -r "$CRUNCH_SRC" /tmp/build/crunch 2>/dev/null
      chmod -R u+w /tmp/build/crunch
      cd /tmp/build/crunch

      # Write .cargo/config.toml for self-build:
      # - Use gcc instead of clang
      # - Use system ld instead of mold
      # - Point to vendored deps
      # Determine library search path for libgcc_s.so.
      GCC_LIB=""
      if [ -n "$MUSL_GCC" ]; then GCC_LIB="$MUSL_GCC/lib"
      elif [ -f "$GCC/lib/libgcc_s.so" ]; then GCC_LIB="$GCC/lib"
      fi

      $BB mkdir -p .cargo
      cat > .cargo/config.toml << CARGOEOF
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/tvlfyi/wu-manber.git"]
git = "https://github.com/tvlfyi/wu-manber.git"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor-deps"

[build]
target = "x86_64-unknown-linux-musl"

[target.x86_64-unknown-linux-musl]
linker = "gcc"
rustflags = ["-C", "link-arg=-Wl,--allow-multiple-definition", "-C", "link-arg=-L${GCC_LIB}"]
CARGOEOF

      echo "=== .cargo/config.toml ==="
      cat .cargo/config.toml

      # Set environment for the Rust build.
      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
      export CC=gcc
      export AR=ar

      # Tell ring to use the from-source gcc for its C/asm compilation.
      export TARGET_CC=gcc
      export TARGET_AR=ar
      export HOST_CC=gcc

      # GCC linker search path for libgcc_s.so (needed by proc-macro .so linking).
      if [ -n "$GCC_LIB" ]; then
        export LIBRARY_PATH="$GCC_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
      fi

      # Debug: verify std lib is findable
      echo "=== Sysroot check ==="
      SYSROOT=$($RUST/bin/rustc --print sysroot 2>&1) || true
      echo "sysroot=$SYSROOT"
      ls "$SYSROOT/lib/rustlib/x86_64-unknown-linux-musl/lib/" 2>&1 | head -5 || echo "no std libs found!"

      # Build for the host target (musl, since rustc itself is musl-linked).
      echo "=== Building crunch ==="
      cargo build \
        --release \
        --locked \
        -j 4 \
        2>&1 || exit 1

      echo "=== Installing ==="
      mkdir -p $out/bin
      cp /tmp/cargo-target/release/crunch $out/bin/

      echo "=== Verify ==="
      ls -la $out/bin/crunch
      file $out/bin/crunch 2>/dev/null || true
      $out/bin/crunch --version 2>/dev/null || $out/bin/crunch --help 2>&1 | head -3
    "%,
  ],
  inputs = [toolchain, gnumake, dash, binutils, musl, gcc, rust, crunch_src],
} | crunch.Derivation
