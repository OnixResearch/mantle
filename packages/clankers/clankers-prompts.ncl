# Build the next low-dependency Clankers external Rust workspace rung with Crunch.
# Generated from packages/clankers/clankers-prompts-source-closure.json.

let crunch = import "../../lib/lib.ncl" in
let seed = import "../../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let rust = import "../../bootstrap/rust.ncl" in
let musl = import "../../bootstrap/musl.ncl" in

let src = crunch.fetchTarball {
  url = "file://packages/clankers/clankers-prompts-src.tar.gz",
  hash = "sha256-9TI3D53BzkMtcvRwQDsxcf0JPGdOuz1/m8zfx6xHClc=",
  name = "clankers-prompts-src",
} in

let crate_0 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/anyhow/anyhow-1.0.102.crate",
  hash = "sha256-whN/nX0EE22vtbaRrj0+VWG52e+FsOX/XSWbL69Lgv4=",
  name = "cargo-crate-anyhow-1.0.102",
} in
let crate_1 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/bitflags/bitflags-2.11.1.crate",
  hash = "sha256-ZoYCxC8CgqGUBqWybeFxLRns8czHaXBdfP4K7IDyw9A=",
  name = "cargo-crate-bitflags-2.11.1",
} in
let crate_2 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/cfg-if/cfg-if-1.0.4.crate",
  hash = "sha256-JOncw/XcmX/jgZbQr0KNQKBa2Kw8dZDnIvBokD95Zm8=",
  name = "cargo-crate-cfg-if-1.0.4",
} in
let crate_3 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/equivalent/equivalent-1.0.2.crate",
  hash = "sha256-uTefUNYrF+m59vVl+AHTdkPVQ6x/qC2rCfMwrE/71VU=",
  name = "cargo-crate-equivalent-1.0.2",
} in
let crate_4 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/errno/errno-0.3.14.crate",
  hash = "sha256-cSXH0CyGEfI547qpgJWFPyMiECmbz01OYahVylWSL+M=",
  name = "cargo-crate-errno-0.3.14",
} in
let crate_5 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/fastrand/fastrand-2.4.1.crate",
  hash = "sha256-RLsLTZmZTjydlfSWTvosjszl9bMHJawS4pADM5jycS8=",
  name = "cargo-crate-fastrand-2.4.1",
} in
let crate_6 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/foldhash/foldhash-0.1.5.crate",
  hash = "sha256-YeUwrWYpIzQrO1ImmpWyKj9VZU925YwAmgm0JFdRPls=",
  name = "cargo-crate-foldhash-0.1.5",
} in
let crate_7 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/getrandom/getrandom-0.4.2.crate",
  hash = "sha256-sdmICWF7BGxEd/kxhqU83XhMn1gsoDVdTUc6So4DhX4=",
  name = "cargo-crate-getrandom-0.4.2",
} in
let crate_8 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/hashbrown/hashbrown-0.15.5.crate",
  hash = "sha256-cmKiV8U6bDL5j6e2N5ZJwrs1tlQTVzIgoLNyfQSPjbg=",
  name = "cargo-crate-hashbrown-0.15.5",
} in
let crate_9 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/hashbrown/hashbrown-0.17.1.crate",
  hash = "sha256-LkjO2+4jThvOnGyuKevw0SRHHbnFAX/PvdsXmHtRKbw=",
  name = "cargo-crate-hashbrown-0.17.1",
} in
let crate_10 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/heck/heck-0.5.0.crate",
  hash = "sha256-vVMnzH1Jabhj7DZIjmlhH1Jdvp9NLxoSyb5eEhFQafU=",
  name = "cargo-crate-heck-0.5.0",
} in
let crate_11 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/id-arena/id-arena-2.3.0.crate",
  hash = "sha256-j/Y1RQSwa38k/QOJc09djWYfPNrIRRnuhG+boT7XKCs=",
  name = "cargo-crate-id-arena-2.3.0",
} in
let crate_12 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/indexmap/indexmap-2.14.0.crate",
  hash = "sha256-EjBfCc8KE8Nap1WfziZolqeZoEEEWdEbSbOSZHX/ZXY=",
  name = "cargo-crate-indexmap-2.14.0",
} in
let crate_13 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/itoa/itoa-1.0.18.crate",
  hash = "sha256-ZGL6/oZCSvfKkpfHOij7AjkgrBqwhd/HN3I+h/9vEoY=",
  name = "cargo-crate-itoa-1.0.18",
} in
let crate_14 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/leb128fmt/leb128fmt-0.1.0.crate",
  hash = "sha256-IS9rw+j5TMjga/DFxBdV0ECnUs0aA2OR7I/dkOxU8TE=",
  name = "cargo-crate-leb128fmt-0.1.0",
} in
let crate_15 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/libc/libc-0.2.186.crate",
  hash = "sha256-FpqYuNmhy4lNAwhLgmVqreTELh/NGcQibzFAhLu1rBg=",
  name = "cargo-crate-libc-0.2.186",
} in
let crate_16 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/linux-raw-sys/linux-raw-sys-0.12.1.crate",
  hash = "sha256-syk6TlGevol6xQAS8HtCnIjDGBcLKcZ762areCAhvGM=",
  name = "cargo-crate-linux-raw-sys-0.12.1",
} in
let crate_17 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/log/log-0.4.29.crate",
  hash = "sha256-YMp2un3Gy++ePbjMSCy0Or79C5iipZUPiXWuqGsMYdE=",
  name = "cargo-crate-log-0.4.29",
} in
let crate_18 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/memchr/memchr-2.8.0.crate",
  hash = "sha256-wi2kXAUuGs4MWgQVXocY2BjAiE1F3tIEX7Ckpu/kSts=",
  name = "cargo-crate-memchr-2.8.0",
} in
let crate_19 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/once_cell/once_cell-1.21.4.crate",
  hash = "sha256-zxpnYD8Plqi8KHQstLpaxBY8efqMEN3CPRuAAgyNtnY=",
  name = "cargo-crate-once_cell-1.21.4",
} in
let crate_20 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/prettyplease/prettyplease-0.2.37.crate",
  hash = "sha256-n8CjW/W4yvufu09Dv54vHtDWmA0mnR8l+GMvBRbzTg8=",
  name = "cargo-crate-prettyplease-0.2.37",
} in
let crate_21 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/proc-macro2/proc-macro2-1.0.106.crate",
  hash = "sha256-q07CrHFPnYwuSZwRZPC9E5xtdGtxTfbzEhWH5y75DXU=",
  name = "cargo-crate-proc-macro2-1.0.106",
} in
let crate_22 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/quote/quote-1.0.45.crate",
  hash = "sha256-szF9be1ikkDq8yvKLmPHh1byCu2anJcPcfrOa9MBOWA=",
  name = "cargo-crate-quote-1.0.45",
} in
let crate_23 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/r-efi/r-efi-6.0.0.crate",
  hash = "sha256-uO4fmFUms1GAdaMHe8TiwWAl+cDbCj53beSXeRzwBR4=",
  name = "cargo-crate-r-efi-6.0.0",
} in
let crate_24 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rustix/rustix-1.1.4.crate",
  hash = "sha256-KOZKGzxdH4rsR64x+NFWb+yLPRSt5vdA3eQ38SI9Jmk=",
  name = "cargo-crate-rustix-1.1.4",
} in
let crate_25 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/semver/semver-1.0.28.crate",
  hash = "sha256-Xlh4qcbYXMpTz82k/lUYLlMwzLl/dRNe2+bTffsxBjI=",
  name = "cargo-crate-semver-1.0.28",
} in
let crate_26 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde/serde-1.0.228.crate",
  hash = "sha256-rqFSEc0HLayISYEkiXwofcH2KX2fiZ/wdV+sdtCJ4AE=",
  name = "cargo-crate-serde-1.0.228",
} in
let crate_27 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_core/serde_core-1.0.228.crate",
  hash = "sha256-GYsEoV8ezYpIQQt/LddKoj17AVZ/jawMRr9nxa8AGcg=",
  name = "cargo-crate-serde_core-1.0.228",
} in
let crate_28 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_derive/serde_derive-1.0.228.crate",
  hash = "sha256-j9x5x1ikvIl2Nnbe7m4of4nL1Op/Y2Gol23+x8dafFc=",
  name = "cargo-crate-serde_derive-1.0.228",
} in
let crate_29 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_json/serde_json-1.0.149.crate",
  hash = "sha256-0RpfHf1sQ/Gu9FFAeJGCWx54vQkq6OKYqLxEyVZRFoo=",
  name = "cargo-crate-serde_json-1.0.149",
} in
let crate_30 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/syn/syn-2.0.117.crate",
  hash = "sha256-DYOdbsK2tyWjvB/GgyZayp2rxt8kpMBwaWbuEay3/0s=",
  name = "cargo-crate-syn-2.0.117",
} in
let crate_31 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/tempfile/tempfile-3.27.0.crate",
  hash = "sha256-GiBxYVHKAzv+cKEHBusB6p7Na1lBYacQodV3Hm6xU9M=",
  name = "cargo-crate-tempfile-3.27.0",
} in
let crate_32 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/unicode-ident/unicode-ident-1.0.24.crate",
  hash = "sha256-49zZGPkl1ce7Yfgzt62FeQdVhgJWu0RBSWwk+JYC1uA=",
  name = "cargo-crate-unicode-ident-1.0.24",
} in
let crate_33 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/unicode-xid/unicode-xid-0.2.6.crate",
  hash = "sha256-915w07Y91OmWx2Irp3gm3oMvYxwF5VNvyb/csgZocGc=",
  name = "cargo-crate-unicode-xid-0.2.6",
} in
let crate_34 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasip2/wasip2-1.0.3+wasi-0.2.9.crate",
  hash = "sha256-fe7U0TLJJFutFAh3WNK9rx/X/RkZtwlDpDV9FFb4s5k=",
  name = "cargo-crate-wasip2-1.0.3-wasi-0.2.9",
} in
let crate_35 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasip3/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06.crate",
  hash = "sha256-qfs8Zyi6SdbiwOIYhyhfX3UHcrwSW/lLUZ9BO5yX9e0=",
  name = "cargo-crate-wasip3-0.4.0-wasi-0.3.0-rc-2026-01-06",
} in
let crate_36 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-encoder/wasm-encoder-0.244.0.crate",
  hash = "sha256-d5GN9w4PDkC1/JQdNhKAAF5GwpIwL4ghNYfRl3X/hyc=",
  name = "cargo-crate-wasm-encoder-0.244.0",
} in
let crate_37 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-metadata/wasm-metadata-0.244.0.crate",
  hash = "sha256-GQd/TeFKVmztxRLpB3zSiw11uxiDKDUl/+hZb0pxAqM=",
  name = "cargo-crate-wasm-metadata-0.244.0",
} in
let crate_38 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasmparser/wasmparser-0.244.0.crate",
  hash = "sha256-pNssMEouAaeI39XUOdtXKY4dJ4SHnwMJo6MSu4Rj7+I=",
  name = "cargo-crate-wasmparser-0.244.0",
} in
let crate_39 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-link/windows-link-0.2.1.crate",
  hash = "sha256-wxBf6m5IAvhw845BDmeeQG+aeNbxrFFmv2AXNg5Gsq8=",
  name = "cargo-crate-windows-link-0.2.1",
} in
let crate_40 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-sys/windows-sys-0.61.2.crate",
  hash = "sha256-u41eiRzZY1jzGpNAswmSx8iL3/V/ni3zAP3qm2fFLVU=",
  name = "cargo-crate-windows-sys-0.61.2",
} in
let crate_41 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen/wit-bindgen-0.51.0.crate",
  hash = "sha256-ICk1Wwb8me35nSkHg0c/1QdnpLc1NjNcKSWj5eww1BY=",
  name = "cargo-crate-wit-bindgen-0.51.0",
} in
let crate_42 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen/wit-bindgen-0.57.1.crate",
  hash = "sha256-WxESOs942/7suutc8AAdZaTW58Zd5O+BWSZRMUpBG6M=",
  name = "cargo-crate-wit-bindgen-0.57.1",
} in
let crate_43 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen-core/wit-bindgen-core-0.51.0.crate",
  hash = "sha256-VUmXadWiWrCOLSV5eCfwWV6IX//a6qfD3XstvOXKQ0o=",
  name = "cargo-crate-wit-bindgen-core-0.51.0",
} in
let crate_44 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen-rust/wit-bindgen-rust-0.51.0.crate",
  hash = "sha256-haEM3RpVgV6QpfWXpPZqZIxuWI7X5HBCiTu9VX8vDYk=",
  name = "cargo-crate-wit-bindgen-rust-0.51.0",
} in
let crate_45 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen-rust-macro/wit-bindgen-rust-macro-0.51.0.crate",
  hash = "sha256-7NGZWd/aOEcXtmjJsPxpzocOSWHrQJPZ566qFSZa2pI=",
  name = "cargo-crate-wit-bindgen-rust-macro-0.51.0",
} in
let crate_46 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-component/wit-component-0.244.0.crate",
  hash = "sha256-5PD0c2YGDBrBOi+WzBRIeCxlQn7pfFbe1io7H60jlRo=",
  name = "cargo-crate-wit-component-0.244.0",
} in
let crate_47 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-parser/wit-parser-0.244.0.crate",
  hash = "sha256-nPJegbTVdoO2Qa+DkWrB6vCDO+V+oOvBNO0Ay6T5Ar0=",
  name = "cargo-crate-wit-parser-0.244.0",
} in
let crate_48 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zmij/zmij-1.0.21.crate",
  hash = "sha256-+AUIi/7jKYvVvDytev43xvNcfQfgDoqxxWSV/k9Da4E=",
  name = "cargo-crate-zmij-1.0.21",
} in

{
  name = "clankers-prompts-0.1.0",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools /tmp/home /tmp/build /tmp/cargo-home /tmp/cargo-target /tmp/vendor
      for cmd in cat mkdir cp chmod ln ls echo rm mv grep tr sort wc expr test basename dirname install find xargs readlink touch true false tee; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"
      export HOME=/tmp/home
      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      export CARGO_NET_OFFLINE=true

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
      for d in $NIX_STORE/*-clankers-prompts-src; do
        if [ -f "$d/Cargo.toml" ]; then SRC="$d"; break; fi
      done
      for tool in TOOLCHAIN MUSL RUST SRC; do
        eval val=\$$tool
        if [ -z "$val" ]; then echo "ERROR: $tool not found" >&2; exit 1; fi
        echo "$tool=$val"
      done

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf "$MUSL/lib/libc.so" /lib/%{seed_dynamic_linker} 2>/dev/null || true

      export PATH="/tmp/tools:$RUST/bin:$TOOLCHAIN/bin"
      export LD_LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export LIBRARY_PATH="$TOOLCHAIN/%{seed_target}/lib"
      export RUSTFLAGS="-C link-arg=-L$TOOLCHAIN/%{seed_target}/lib"

      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-anyhow-1.0.102; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: anyhow-1.0.102" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/anyhow-1.0.102"
      cp -r "$CRATE_SRC/." "/tmp/vendor/anyhow-1.0.102/"
      cat > "/tmp/vendor/anyhow-1.0.102/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"7f202df86484c868dbad7eaa557ef785d5c66295e41b460ef922eca0723b842c"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-bitflags-2.11.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: bitflags-2.11.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/bitflags-2.11.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/bitflags-2.11.1/"
      cat > "/tmp/vendor/bitflags-2.11.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"c4512299f36f043ab09a583e57bceb5a5aab7a73db1805848e8fef3c9e8c78b3"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-cfg-if-1.0.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: cfg-if-1.0.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/cfg-if-1.0.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/cfg-if-1.0.4/"
      cat > "/tmp/vendor/cfg-if-1.0.4/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-equivalent-1.0.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: equivalent-1.0.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/equivalent-1.0.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/equivalent-1.0.2/"
      cat > "/tmp/vendor/equivalent-1.0.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-errno-0.3.14; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: errno-0.3.14" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/errno-0.3.14"
      cp -r "$CRATE_SRC/." "/tmp/vendor/errno-0.3.14/"
      cat > "/tmp/vendor/errno-0.3.14/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-fastrand-2.4.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: fastrand-2.4.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/fastrand-2.4.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/fastrand-2.4.1/"
      cat > "/tmp/vendor/fastrand-2.4.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9f1f227452a390804cdb637b74a86990f2a7d7ba4b7d5693aac9b4dd6defd8d6"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-foldhash-0.1.5; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: foldhash-0.1.5" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/foldhash-0.1.5"
      cp -r "$CRATE_SRC/." "/tmp/vendor/foldhash-0.1.5/"
      cat > "/tmp/vendor/foldhash-0.1.5/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-getrandom-0.4.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: getrandom-0.4.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/getrandom-0.4.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/getrandom-0.4.2/"
      cat > "/tmp/vendor/getrandom-0.4.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"0de51e6874e94e7bf76d726fc5d13ba782deca734ff60d5bb2fb2607c7406555"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-hashbrown-0.15.5; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: hashbrown-0.15.5" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/hashbrown-0.15.5"
      cp -r "$CRATE_SRC/." "/tmp/vendor/hashbrown-0.15.5/"
      cat > "/tmp/vendor/hashbrown-0.15.5/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-hashbrown-0.17.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: hashbrown-0.17.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/hashbrown-0.17.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/hashbrown-0.17.1/"
      cat > "/tmp/vendor/hashbrown-0.17.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-heck-0.5.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: heck-0.5.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/heck-0.5.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/heck-0.5.0/"
      cat > "/tmp/vendor/heck-0.5.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-id-arena-2.3.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: id-arena-2.3.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/id-arena-2.3.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/id-arena-2.3.0/"
      cat > "/tmp/vendor/id-arena-2.3.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"3d3067d79b975e8844ca9eb072e16b31c3c1c36928edf9c6789548c524d0d954"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-indexmap-2.14.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: indexmap-2.14.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/indexmap-2.14.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/indexmap-2.14.0/"
      cat > "/tmp/vendor/indexmap-2.14.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-itoa-1.0.18; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: itoa-1.0.18" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/itoa-1.0.18"
      cp -r "$CRATE_SRC/." "/tmp/vendor/itoa-1.0.18/"
      cat > "/tmp/vendor/itoa-1.0.18/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-leb128fmt-0.1.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: leb128fmt-0.1.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/leb128fmt-0.1.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/leb128fmt-0.1.0/"
      cat > "/tmp/vendor/leb128fmt-0.1.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"09edd9e8b54e49e587e4f6295a7d29c3ea94d469cb40ab8ca70b288248a81db2"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-libc-0.2.186; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: libc-0.2.186" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/libc-0.2.186"
      cp -r "$CRATE_SRC/." "/tmp/vendor/libc-0.2.186/"
      cat > "/tmp/vendor/libc-0.2.186/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"68ab91017fe16c622486840e4c83c9a37afeff978bd239b5293d61ece587de66"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-linux-raw-sys-0.12.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: linux-raw-sys-0.12.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/linux-raw-sys-0.12.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/linux-raw-sys-0.12.1/"
      cat > "/tmp/vendor/linux-raw-sys-0.12.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-log-0.4.29; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: log-0.4.29" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/log-0.4.29"
      cp -r "$CRATE_SRC/." "/tmp/vendor/log-0.4.29/"
      cat > "/tmp/vendor/log-0.4.29/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"5e5032e24019045c762d3c0f28f5b6b8bbf38563a65908389bf7978758920897"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-memchr-2.8.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: memchr-2.8.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/memchr-2.8.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/memchr-2.8.0/"
      cat > "/tmp/vendor/memchr-2.8.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"f8ca58f447f06ed17d5fc4043ce1b10dd205e060fb3ce5b979b8ed8e59ff3f79"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-once_cell-1.21.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: once_cell-1.21.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/once_cell-1.21.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/once_cell-1.21.4/"
      cat > "/tmp/vendor/once_cell-1.21.4/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-prettyplease-0.2.37; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: prettyplease-0.2.37" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/prettyplease-0.2.37"
      cp -r "$CRATE_SRC/." "/tmp/vendor/prettyplease-0.2.37/"
      cat > "/tmp/vendor/prettyplease-0.2.37/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-proc-macro2-1.0.106; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: proc-macro2-1.0.106" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/proc-macro2-1.0.106"
      cp -r "$CRATE_SRC/." "/tmp/vendor/proc-macro2-1.0.106/"
      cat > "/tmp/vendor/proc-macro2-1.0.106/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-quote-1.0.45; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: quote-1.0.45" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/quote-1.0.45"
      cp -r "$CRATE_SRC/." "/tmp/vendor/quote-1.0.45/"
      cat > "/tmp/vendor/quote-1.0.45/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"41f2619966050689382d2b44f664f4bc593e129785a36d6ee376ddf37259b924"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-r-efi-6.0.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: r-efi-6.0.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/r-efi-6.0.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/r-efi-6.0.0/"
      cat > "/tmp/vendor/r-efi-6.0.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"f8dcc9c7d52a811697d2151c701e0d08956f92b0e24136cf4cf27b57a6a0d9bf"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rustix-1.1.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rustix-1.1.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rustix-1.1.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rustix-1.1.4/"
      cat > "/tmp/vendor/rustix-1.1.4/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-semver-1.0.28; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: semver-1.0.28" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/semver-1.0.28"
      cp -r "$CRATE_SRC/." "/tmp/vendor/semver-1.0.28/"
      cat > "/tmp/vendor/semver-1.0.28/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde-1.0.228/"
      cat > "/tmp/vendor/serde-1.0.228/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_core-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_core-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_core-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_core-1.0.228/"
      cat > "/tmp/vendor/serde_core-1.0.228/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_derive-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_derive-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_derive-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_derive-1.0.228/"
      cat > "/tmp/vendor/serde_derive-1.0.228/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_json-1.0.149; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_json-1.0.149" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_json-1.0.149"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_json-1.0.149/"
      cat > "/tmp/vendor/serde_json-1.0.149/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"83fc039473c5595ace860d8c4fafa220ff474b3fc6bfdb4293327f1a37e94d86"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-syn-2.0.117; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: syn-2.0.117" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/syn-2.0.117"
      cp -r "$CRATE_SRC/." "/tmp/vendor/syn-2.0.117/"
      cat > "/tmp/vendor/syn-2.0.117/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"e665b8803e7b1d2a727f4023456bbbbe74da67099c585258af0ad9c5013b9b99"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-tempfile-3.27.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: tempfile-3.27.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/tempfile-3.27.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/tempfile-3.27.0/"
      cat > "/tmp/vendor/tempfile-3.27.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-unicode-ident-1.0.24; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: unicode-ident-1.0.24" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/unicode-ident-1.0.24"
      cp -r "$CRATE_SRC/." "/tmp/vendor/unicode-ident-1.0.24/"
      cat > "/tmp/vendor/unicode-ident-1.0.24/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-unicode-xid-0.2.6; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: unicode-xid-0.2.6" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/unicode-xid-0.2.6"
      cp -r "$CRATE_SRC/." "/tmp/vendor/unicode-xid-0.2.6/"
      cat > "/tmp/vendor/unicode-xid-0.2.6/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"ebc1c04c71510c7f702b52b7c350734c9ff1295c464a03335b00bb84fc54f853"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasip2-1.0.3-wasi-0.2.9; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasip2-1.0.3+wasi-0.2.9" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasip2-1.0.3+wasi-0.2.9"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasip2-1.0.3+wasi-0.2.9/"
      cat > "/tmp/vendor/wasip2-1.0.3+wasi-0.2.9/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"20064672db26d7cdc89c7798c48a0fdfac8213434a1186e5ef29fd560ae223d6"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasip3-0.4.0-wasi-0.3.0-rc-2026-01-06; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06/"
      cat > "/tmp/vendor/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"5428f8bf88ea5ddc08faddef2ac4a67e390b88186c703ce6dbd955e1c145aca5"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-encoder-0.244.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-encoder-0.244.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-encoder-0.244.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-encoder-0.244.0/"
      cat > "/tmp/vendor/wasm-encoder-0.244.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"990065f2fe63003fe337b932cfb5e3b80e0b4d0f5ff650e6985b1048f62c8319"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-metadata-0.244.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-metadata-0.244.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-metadata-0.244.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-metadata-0.244.0/"
      cat > "/tmp/vendor/wasm-metadata-0.244.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"bb0e353e6a2fbdc176932bbaab493762eb1255a7900fe0fea1a2f96c296cc909"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasmparser-0.244.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasmparser-0.244.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasmparser-0.244.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasmparser-0.244.0/"
      cat > "/tmp/vendor/wasmparser-0.244.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"47b807c72e1bac69382b3a6fb3dbe8ea4c0ed87ff5629b8685ae6b9a611028fe"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-link-0.2.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-link-0.2.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-link-0.2.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-link-0.2.1/"
      cat > "/tmp/vendor/windows-link-0.2.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-sys-0.61.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-sys-0.61.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-sys-0.61.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-sys-0.61.2/"
      cat > "/tmp/vendor/windows-sys-0.61.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-0.51.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-0.51.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-0.51.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-0.51.0/"
      cat > "/tmp/vendor/wit-bindgen-0.51.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d7249219f66ced02969388cf2bb044a09756a083d0fab1e566056b04d9fbcaa5"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-0.57.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-0.57.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-0.57.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-0.57.1/"
      cat > "/tmp/vendor/wit-bindgen-0.57.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-core-0.51.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-core-0.51.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-core-0.51.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-core-0.51.0/"
      cat > "/tmp/vendor/wit-bindgen-core-0.51.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"ea61de684c3ea68cb082b7a88508a8b27fcc8b797d738bfc99a82facf1d752dc"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-rust-0.51.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-rust-0.51.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-rust-0.51.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-rust-0.51.0/"
      cat > "/tmp/vendor/wit-bindgen-rust-0.51.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"b7c566e0f4b284dd6561c786d9cb0142da491f46a9fbed79ea69cdad5db17f21"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-rust-macro-0.51.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-rust-macro-0.51.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-rust-macro-0.51.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-rust-macro-0.51.0/"
      cat > "/tmp/vendor/wit-bindgen-rust-macro-0.51.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"0c0f9bfd77e6a48eccf51359e3ae77140a7f50b1e2ebfe62422d8afdaffab17a"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-component-0.244.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-component-0.244.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-component-0.244.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-component-0.244.0/"
      cat > "/tmp/vendor/wit-component-0.244.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"9d66ea20e9553b30172b5e831994e35fbde2d165325bec84fc43dbf6f4eb9cb2"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-parser-0.244.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-parser-0.244.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-parser-0.244.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-parser-0.244.0/"
      cat > "/tmp/vendor/wit-parser-0.244.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"ecc8ac4bc1dc3381b7f59c34f00b67e18f910c2c0f50015669dde7def656a736"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zmij-1.0.21; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zmij-1.0.21" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zmij-1.0.21"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zmij-1.0.21/"
      cat > "/tmp/vendor/zmij-1.0.21/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"b8848ee67ecc8aedbaf3e4122217aff892639231befc6a1b58d29fff4c2cabaa"}
CHECKSUMEOF

      cp -r "$SRC/." /tmp/build/
      chmod -R u+w /tmp/build /tmp/vendor
      cd /tmp/build
      mkdir -p .cargo
      cat > .cargo/config.toml << CARGOEOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/tmp/vendor"

[target.x86_64-unknown-linux-musl]
linker = "%{seed_target}-gcc"

[net]
offline = true
CARGOEOF

      cargo build --release --locked --offline -p clankers-prompts --target x86_64-unknown-linux-musl

      mkdir -p $out/lib $out/share/clankers
      cp $CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/libclankers_prompts.rlib $out/lib/
      cat > $out/share/clankers/clankers-prompts-build-receipt.txt << RECEIPT
package=clankers-prompts
cargo_command=cargo build --release --locked --offline -p clankers-prompts --target x86_64-unknown-linux-musl
source_archive_sha256=b4a24af9e442414396ffa46818c9d04892a7dde12d08489b18fd89f4be4fdd66
registry_crate_count=49
artifact=$out/lib/libclankers_prompts.rlib
RECEIPT
      test -s $out/lib/libclankers_prompts.rlib
    "%,
  ],
  inputs = [toolchain, musl, rust, src, crate_0, crate_1, crate_2, crate_3, crate_4, crate_5, crate_6, crate_7, crate_8, crate_9, crate_10, crate_11, crate_12, crate_13, crate_14, crate_15, crate_16, crate_17, crate_18, crate_19, crate_20, crate_21, crate_22, crate_23, crate_24, crate_25, crate_26, crate_27, crate_28, crate_29, crate_30, crate_31, crate_32, crate_33, crate_34, crate_35, crate_36, crate_37, crate_38, crate_39, crate_40, crate_41, crate_42, crate_43, crate_44, crate_45, crate_46, crate_47, crate_48],
} | crunch.Derivation
