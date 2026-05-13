# Build the first Clankers external Rust workspace rung with Crunch.
# Generated from packages/clankers/clanker-message-source-closure.json.

let crunch = import "../../lib/lib.ncl" in
let seed = import "../../bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let rust = import "../../bootstrap/rust.ncl" in
let musl = import "../../bootstrap/musl.ncl" in

let src = crunch.fetchTarball {
  url = "file://packages/clankers/clanker-message-src.tar.gz",
  hash = "sha256-bM5GH7LXek+EhxqRJH7P/2zh8hKAQFbjDc5QpP8sinY=",
  name = "clanker-message-src",
} in

let crate_0 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/android_system_properties/android_system_properties-0.1.5.crate",
  hash = "sha256-AVSo7h3ytu8IwFL9zJpyFvh7lLSztIP0W29KL00v/2k=",
  name = "cargo-crate-android_system_properties-0.1.5",
} in
let crate_1 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/autocfg/autocfg-1.5.0.crate",
  hash = "sha256-NhFHtHDmMRNf86xuOOMYXVv4DHdN0dBi9JzOQQDysEY=",
  name = "cargo-crate-autocfg-1.5.0",
} in
let crate_2 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/bumpalo/bumpalo-3.20.2.crate",
  hash = "sha256-vmGQgtQlgYXm+17sl5F+Q63Xr7zrGUtRuWRmVuVwiqc=",
  name = "cargo-crate-bumpalo-3.20.2",
} in
let crate_3 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/cc/cc-1.2.62.crate",
  hash = "sha256-ek4kwZnw8qjqRVZGnqPJRT3pG0mYiEKXn5PRjBZQk5o=",
  name = "cargo-crate-cc-1.2.62",
} in
let crate_4 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/cfg-if/cfg-if-1.0.4.crate",
  hash = "sha256-JOncw/XcmX/jgZbQr0KNQKBa2Kw8dZDnIvBokD95Zm8=",
  name = "cargo-crate-cfg-if-1.0.4",
} in
let crate_5 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/chrono/chrono-0.4.44.crate",
  hash = "sha256-GE6nQw3C9goxM3j7eH+xq8BkQATil9YANa3uCKbSCrg=",
  name = "cargo-crate-chrono-0.4.44",
} in
let crate_6 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/core-foundation-sys/core-foundation-sys-0.8.7.crate",
  hash = "sha256-J98GeYhncf4Vsuzy7qsbeFz1RTgBLzwez0/CNpR3ncA=",
  name = "cargo-crate-core-foundation-sys-0.8.7",
} in
let crate_7 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/diff/diff-0.1.13.crate",
  hash = "sha256-ry/vPEI3ztO9d2L2poRUhrh3YP5/nWY/RoCw/Lj+Ja8=",
  name = "cargo-crate-diff-0.1.13",
} in
let crate_8 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/find-msvc-tools/find-msvc-tools-0.1.9.crate",
  hash = "sha256-CAXurpZL1ltJrnUbTwp7EX5MKxL4gNTYKOyYnG8YrrQ=",
  name = "cargo-crate-find-msvc-tools-0.1.9",
} in
let crate_9 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-core/futures-core-0.3.32.crate",
  hash = "sha256-nOdwbV25HUWT3CfhPuM++GI/1LvyG0qAHUxyj+45FyM=",
  name = "cargo-crate-futures-core-0.3.32",
} in
let crate_10 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-task/futures-task-0.3.32.crate",
  hash = "sha256-qnfdK4NuvgEMR2/veqavH3aM2jwh3Hyjf2/M9EhNTJI=",
  name = "cargo-crate-futures-task-0.3.32",
} in
let crate_11 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-util/futures-util-0.3.32.crate",
  hash = "sha256-SiO+qJuQIbIabm+A1qKOCaMKXzlqOYmV1kOr+F0ohGw=",
  name = "cargo-crate-futures-util-0.3.32",
} in
let crate_12 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/getrandom/getrandom-0.3.4.crate",
  hash = "sha256-cFgw4woE4NyheMo3kiTnmP5N2N598UbcEgknPPFIOHY=",
  name = "cargo-crate-getrandom-0.3.4",
} in
let crate_13 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/hex/hex-0.4.3.crate",
  hash = "sha256-llCsvm54GCeIPOcSqNeASYy3BqViXYjfeddcD46a7xM=",
  name = "cargo-crate-hex-0.4.3",
} in
let crate_14 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/iana-time-zone/iana-time-zone-0.1.65.crate",
  hash = "sha256-96ALKuOIdwUp1ToTazS2msNuc8QqX9vcz0BaEfF8bMQ=",
  name = "cargo-crate-iana-time-zone-0.1.65",
} in
let crate_15 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/iana-time-zone-haiku/iana-time-zone-haiku-0.1.2.crate",
  hash = "sha256-LX0PMxjGBFiZ1gDa13OK+IuhNSpjEbfD/d3rrkQL2q8=",
  name = "cargo-crate-iana-time-zone-haiku-0.1.2",
} in
let crate_16 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/itoa/itoa-1.0.18.crate",
  hash = "sha256-ZGL6/oZCSvfKkpfHOij7AjkgrBqwhd/HN3I+h/9vEoY=",
  name = "cargo-crate-itoa-1.0.18",
} in
let crate_17 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/js-sys/js-sys-0.3.98.crate",
  hash = "sha256-3q0tVeRl07jJsgyZ3grWNxUXmoVg8JPyXpZC4wgsFMw=",
  name = "cargo-crate-js-sys-0.3.98",
} in
let crate_18 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/libc/libc-0.2.186.crate",
  hash = "sha256-FpqYuNmhy4lNAwhLgmVqreTELh/NGcQibzFAhLu1rBg=",
  name = "cargo-crate-libc-0.2.186",
} in
let crate_19 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/log/log-0.4.29.crate",
  hash = "sha256-YMp2un3Gy++ePbjMSCy0Or79C5iipZUPiXWuqGsMYdE=",
  name = "cargo-crate-log-0.4.29",
} in
let crate_20 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/memchr/memchr-2.8.0.crate",
  hash = "sha256-wi2kXAUuGs4MWgQVXocY2BjAiE1F3tIEX7Ckpu/kSts=",
  name = "cargo-crate-memchr-2.8.0",
} in
let crate_21 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/num-traits/num-traits-0.2.19.crate",
  hash = "sha256-5L0mKAem8o1xs1FxPfdG39Eiogz5oKWWzEuYz1nJFUk=",
  name = "cargo-crate-num-traits-0.2.19",
} in
let crate_22 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/once_cell/once_cell-1.21.4.crate",
  hash = "sha256-zxpnYD8Plqi8KHQstLpaxBY8efqMEN3CPRuAAgyNtnY=",
  name = "cargo-crate-once_cell-1.21.4",
} in
let crate_23 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/pin-project-lite/pin-project-lite-0.2.17.crate",
  hash = "sha256-AHE+rKJhLXKKHwVRyu7DpP8czhYJMTbIdgpHicpTJlQ=",
  name = "cargo-crate-pin-project-lite-0.2.17",
} in
let crate_24 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/ppv-lite86/ppv-lite86-0.2.21.crate",
  hash = "sha256-vtc6gNA6gzRaV8GyDXnBjaNh1ux0iZvEgxHqRCOHodw=",
  name = "cargo-crate-ppv-lite86-0.2.21",
} in
let crate_25 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/pretty_assertions/pretty_assertions-1.4.1.crate",
  hash = "sha256-FxHNoHA119cyf7t96S4PHbGMU4HwitPFmzkEumFgA2I=",
  name = "cargo-crate-pretty_assertions-1.4.1",
} in
let crate_26 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/proc-macro2/proc-macro2-1.0.106.crate",
  hash = "sha256-q07CrHFPnYwuSZwRZPC9E5xtdGtxTfbzEhWH5y75DXU=",
  name = "cargo-crate-proc-macro2-1.0.106",
} in
let crate_27 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/quote/quote-1.0.45.crate",
  hash = "sha256-szF9be1ikkDq8yvKLmPHh1byCu2anJcPcfrOa9MBOWA=",
  name = "cargo-crate-quote-1.0.45",
} in
let crate_28 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/r-efi/r-efi-5.3.0.crate",
  hash = "sha256-7GkIPnWzp+SYzzhAqzQYilnLagps+2djhdoa+ye7xfk=",
  name = "cargo-crate-r-efi-5.3.0",
} in
let crate_29 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand/rand-0.9.4.crate",
  hash = "sha256-WCQ/qde3lEPznHSuxFklCCGifAUyBjnIBbUAt2rewkM=",
  name = "cargo-crate-rand-0.9.4",
} in
let crate_30 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand_chacha/rand_chacha-0.9.0.crate",
  hash = "sha256-Wv2V1lO+3qlnNRkbNDOXK4S6aYWQi9fMIyfHEOFdjms=",
  name = "cargo-crate-rand_chacha-0.9.0",
} in
let crate_31 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand_core/rand_core-0.9.5.crate",
  hash = "sha256-dyTUcdW5kUXQd1q1T8FqTBPNTGzguOub627kALPegf0=",
  name = "cargo-crate-rand_core-0.9.5",
} in
let crate_32 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rustversion/rustversion-1.0.22.crate",
  hash = "sha256-1Ejjt+8zEI5SXTBTVh0pK3LGNeq94QdAncmBu9CBeRw=",
  name = "cargo-crate-rustversion-1.0.22",
} in
let crate_33 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde/serde-1.0.228.crate",
  hash = "sha256-rqFSEc0HLayISYEkiXwofcH2KX2fiZ/wdV+sdtCJ4AE=",
  name = "cargo-crate-serde-1.0.228",
} in
let crate_34 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_core/serde_core-1.0.228.crate",
  hash = "sha256-GYsEoV8ezYpIQQt/LddKoj17AVZ/jawMRr9nxa8AGcg=",
  name = "cargo-crate-serde_core-1.0.228",
} in
let crate_35 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_derive/serde_derive-1.0.228.crate",
  hash = "sha256-j9x5x1ikvIl2Nnbe7m4of4nL1Op/Y2Gol23+x8dafFc=",
  name = "cargo-crate-serde_derive-1.0.228",
} in
let crate_36 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_json/serde_json-1.0.149.crate",
  hash = "sha256-0RpfHf1sQ/Gu9FFAeJGCWx54vQkq6OKYqLxEyVZRFoo=",
  name = "cargo-crate-serde_json-1.0.149",
} in
let crate_37 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/shlex/shlex-1.3.0.crate",
  hash = "sha256-HoJHyJYByCZfnRnpmXtwWVqATF+nx9/lQpnF+pbAZdw=",
  name = "cargo-crate-shlex-1.3.0",
} in
let crate_38 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/slab/slab-0.4.12.crate",
  hash = "sha256-lmYIBXDcJh7sxnh47aSMbJgX6NJq1JWggCqnpmeWCdM=",
  name = "cargo-crate-slab-0.4.12",
} in
let crate_39 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/syn/syn-2.0.117.crate",
  hash = "sha256-DYOdbsK2tyWjvB/GgyZayp2rxt8kpMBwaWbuEay3/0s=",
  name = "cargo-crate-syn-2.0.117",
} in
let crate_40 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/unicode-ident/unicode-ident-1.0.24.crate",
  hash = "sha256-49zZGPkl1ce7Yfgzt62FeQdVhgJWu0RBSWwk+JYC1uA=",
  name = "cargo-crate-unicode-ident-1.0.24",
} in
let crate_41 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasip2/wasip2-1.0.3+wasi-0.2.9.crate",
  hash = "sha256-fe7U0TLJJFutFAh3WNK9rx/X/RkZtwlDpDV9FFb4s5k=",
  name = "cargo-crate-wasip2-1.0.3-wasi-0.2.9",
} in
let crate_42 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen/wasm-bindgen-0.2.121.crate",
  hash = "sha256-tXCsKPY4I/S8o4Aa2FH2o33or+J6wJ6RbAJH6SKGD0w=",
  name = "cargo-crate-wasm-bindgen-0.2.121",
} in
let crate_43 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-macro/wasm-bindgen-macro-0.2.121.crate",
  hash = "sha256-LO0jwsiQs7mFVjfcJGdAWt065/egUl1eH7K7zOqexMo=",
  name = "cargo-crate-wasm-bindgen-macro-0.2.121",
} in
let crate_44 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-macro-support/wasm-bindgen-macro-support-0.2.121.crate",
  hash = "sha256-wpDaftiPHegiNzU/6aU0fQ+hhyP+JQkII6TgDeqljK0=",
  name = "cargo-crate-wasm-bindgen-macro-support-0.2.121",
} in
let crate_45 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-shared/wasm-bindgen-shared-0.2.121.crate",
  hash = "sha256-lpeGK4tFbYsUuCzbiETCxhRtUaKbizeleLE9hTRa9qg=",
  name = "cargo-crate-wasm-bindgen-shared-0.2.121",
} in
let crate_46 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-core/windows-core-0.62.2.crate",
  hash = "sha256-k7frSWB+Z7LgqrLEOBf8HhxU3l0l4B2uE9lpyrxWugU=",
  name = "cargo-crate-windows-core-0.62.2",
} in
let crate_47 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-implement/windows-implement-0.60.2.crate",
  hash = "sha256-M2HoWN4dIvomaew4Z+S6PjiTiGikuxXn1ywLJAHT97I=",
  name = "cargo-crate-windows-implement-0.60.2",
} in
let crate_48 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-interface/windows-interface-0.59.3.crate",
  hash = "sha256-mKXdXwNnqUBljPXNcZST+QsIyWcpv7D1ScyKTYWyxwU=",
  name = "cargo-crate-windows-interface-0.59.3",
} in
let crate_49 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-link/windows-link-0.2.1.crate",
  hash = "sha256-wxBf6m5IAvhw845BDmeeQG+aeNbxrFFmv2AXNg5Gsq8=",
  name = "cargo-crate-windows-link-0.2.1",
} in
let crate_50 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-result/windows-result-0.4.1.crate",
  hash = "sha256-BLuJwKRiDEK+WoJNT0aUPYTTa+DOGKRdUTtB6jbZSgc=",
  name = "cargo-crate-windows-result-0.4.1",
} in
let crate_51 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-strings/windows-strings-0.5.1.crate",
  hash = "sha256-BfZdwIfLZyutF12eavKArT2KIU5Do4QEfu5OuMks1A8=",
  name = "cargo-crate-windows-strings-0.5.1",
} in
let crate_52 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen/wit-bindgen-0.57.1.crate",
  hash = "sha256-WxESOs942/7suutc8AAdZaTW58Zd5O+BWSZRMUpBG6M=",
  name = "cargo-crate-wit-bindgen-0.57.1",
} in
let crate_53 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/yansi/yansi-1.0.1.crate",
  hash = "sha256-tlikTLm/MWVQRc6kUh3CxkPGCBZKqz7vQJVB7BoUGfU=",
  name = "cargo-crate-yansi-1.0.1",
} in
let crate_54 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zerocopy/zerocopy-0.8.48.crate",
  hash = "sha256-EBo4fZE8DP9bK4fyIjg4fvFxq8dFFHb3Sev8goO6YPA=",
  name = "cargo-crate-zerocopy-0.8.48",
} in
let crate_55 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zerocopy-derive/zerocopy-derive-0.8.48.crate",
  hash = "sha256-aULMyDBEeMYmsToFn1RLXidLwPEkfOfwGQkVX1HLzUE=",
  name = "cargo-crate-zerocopy-derive-0.8.48",
} in
let crate_56 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zmij/zmij-1.0.21.crate",
  hash = "sha256-+AUIi/7jKYvVvDytev43xvNcfQfgDoqxxWSV/k9Da4E=",
  name = "cargo-crate-zmij-1.0.21",
} in

{
  name = "clanker-message-0.1.0",
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
      for d in $NIX_STORE/*-clanker-message-src; do
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
      for d in $NIX_STORE/*-cargo-crate-android_system_properties-0.1.5; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: android_system_properties-0.1.5" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/android_system_properties-0.1.5"
      cp -r "$CRATE_SRC/." "/tmp/vendor/android_system_properties-0.1.5/"
      cat > "/tmp/vendor/android_system_properties-0.1.5/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-autocfg-1.5.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: autocfg-1.5.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/autocfg-1.5.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/autocfg-1.5.0/"
      cat > "/tmp/vendor/autocfg-1.5.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-bumpalo-3.20.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: bumpalo-3.20.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/bumpalo-3.20.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/bumpalo-3.20.2/"
      cat > "/tmp/vendor/bumpalo-3.20.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"5d20789868f4b01b2f2caec9f5c4e0213b41e3e5702a50157d699ae31ced2fcb"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-cc-1.2.62; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: cc-1.2.62" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/cc-1.2.62"
      cp -r "$CRATE_SRC/." "/tmp/vendor/cc-1.2.62/"
      cat > "/tmp/vendor/cc-1.2.62/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"a1dce859f0832a7d088c4f1119888ab94ef4b5d6795d1ce05afb7fe159d79f98"}
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
      for d in $NIX_STORE/*-cargo-crate-chrono-0.4.44; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: chrono-0.4.44" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/chrono-0.4.44"
      cp -r "$CRATE_SRC/." "/tmp/vendor/chrono-0.4.44/"
      cat > "/tmp/vendor/chrono-0.4.44/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"c673075a2e0e5f4a1dde27ce9dee1ea4558c7ffe648f576438a20ca1d2acc4b0"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-core-foundation-sys-0.8.7; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: core-foundation-sys-0.8.7" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/core-foundation-sys-0.8.7"
      cp -r "$CRATE_SRC/." "/tmp/vendor/core-foundation-sys-0.8.7/"
      cat > "/tmp/vendor/core-foundation-sys-0.8.7/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-diff-0.1.13; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: diff-0.1.13" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/diff-0.1.13"
      cp -r "$CRATE_SRC/." "/tmp/vendor/diff-0.1.13/"
      cat > "/tmp/vendor/diff-0.1.13/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"56254986775e3233ffa9c4d7d3faaf6d36a2c09d30b20687e9f88bc8bafc16c8"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-find-msvc-tools-0.1.9; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: find-msvc-tools-0.1.9" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/find-msvc-tools-0.1.9"
      cp -r "$CRATE_SRC/." "/tmp/vendor/find-msvc-tools-0.1.9/"
      cat > "/tmp/vendor/find-msvc-tools-0.1.9/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"5baebc0774151f905a1a2cc41989300b1e6fbb29aff0ceffa1064fdd3088d582"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-core-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-core-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-core-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-core-0.3.32/"
      cat > "/tmp/vendor/futures-core-0.3.32/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"7e3450815272ef58cec6d564423f6e755e25379b217b0bc688e295ba24df6b1d"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-task-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-task-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-task-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-task-0.3.32/"
      cat > "/tmp/vendor/futures-task-0.3.32/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"037711b3d59c33004d3856fbdc83b99d4ff37a24768fa1be9ce3538a1cde4393"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-util-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-util-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-util-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-util-0.3.32/"
      cat > "/tmp/vendor/futures-util-0.3.32/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"389ca41296e6190b48053de0321d02a77f32f8a5d2461dd38762c0593805c6d6"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-getrandom-0.3.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: getrandom-0.3.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/getrandom-0.3.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/getrandom-0.3.4/"
      cat > "/tmp/vendor/getrandom-0.3.4/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-hex-0.4.3; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: hex-0.4.3" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/hex-0.4.3"
      cp -r "$CRATE_SRC/." "/tmp/vendor/hex-0.4.3/"
      cat > "/tmp/vendor/hex-0.4.3/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-iana-time-zone-0.1.65; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: iana-time-zone-0.1.65" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/iana-time-zone-0.1.65"
      cp -r "$CRATE_SRC/." "/tmp/vendor/iana-time-zone-0.1.65/"
      cat > "/tmp/vendor/iana-time-zone-0.1.65/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"e31bc9ad994ba00e440a8aa5c9ef0ec67d5cb5e5cb0cc7f8b744a35b389cc470"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-iana-time-zone-haiku-0.1.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: iana-time-zone-haiku-0.1.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/iana-time-zone-haiku-0.1.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/iana-time-zone-haiku-0.1.2/"
      cat > "/tmp/vendor/iana-time-zone-haiku-0.1.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f"}
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
      for d in $NIX_STORE/*-cargo-crate-js-sys-0.3.98; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: js-sys-0.3.98" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/js-sys-0.3.98"
      cp -r "$CRATE_SRC/." "/tmp/vendor/js-sys-0.3.98/"
      cat > "/tmp/vendor/js-sys-0.3.98/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"67df7112613f8bfd9150013a0314e196f4800d3201ae742489d999db2f979f08"}
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
      for d in $NIX_STORE/*-cargo-crate-num-traits-0.2.19; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: num-traits-0.2.19" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/num-traits-0.2.19"
      cp -r "$CRATE_SRC/." "/tmp/vendor/num-traits-0.2.19/"
      cat > "/tmp/vendor/num-traits-0.2.19/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"}
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
      for d in $NIX_STORE/*-cargo-crate-pin-project-lite-0.2.17; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: pin-project-lite-0.2.17" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/pin-project-lite-0.2.17"
      cp -r "$CRATE_SRC/." "/tmp/vendor/pin-project-lite-0.2.17/"
      cat > "/tmp/vendor/pin-project-lite-0.2.17/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-ppv-lite86-0.2.21; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: ppv-lite86-0.2.21" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/ppv-lite86-0.2.21"
      cp -r "$CRATE_SRC/." "/tmp/vendor/ppv-lite86-0.2.21/"
      cat > "/tmp/vendor/ppv-lite86-0.2.21/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-pretty_assertions-1.4.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: pretty_assertions-1.4.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/pretty_assertions-1.4.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/pretty_assertions-1.4.1/"
      cat > "/tmp/vendor/pretty_assertions-1.4.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"3ae130e2f271fbc2ac3a40fb1d07180839cdbbe443c7a27e1e3c13c5cac0116d"}
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
      for d in $NIX_STORE/*-cargo-crate-r-efi-5.3.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: r-efi-5.3.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/r-efi-5.3.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/r-efi-5.3.0/"
      cat > "/tmp/vendor/r-efi-5.3.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand-0.9.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand-0.9.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand-0.9.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand-0.9.4/"
      cat > "/tmp/vendor/rand-0.9.4/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"44c5af06bb1b7d3216d91932aed5265164bf384dc89cd6ba05cf59a35f5f76ea"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand_chacha-0.9.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand_chacha-0.9.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand_chacha-0.9.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand_chacha-0.9.0/"
      cat > "/tmp/vendor/rand_chacha-0.9.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand_core-0.9.5; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand_core-0.9.5" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand_core-0.9.5"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand_core-0.9.5/"
      cat > "/tmp/vendor/rand_core-0.9.5/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"76afc826de14238e6e8c374ddcc1fa19e374fd8dd986b0d2af0d02377261d83c"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rustversion-1.0.22; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rustversion-1.0.22" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rustversion-1.0.22"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rustversion-1.0.22/"
      cat > "/tmp/vendor/rustversion-1.0.22/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d"}
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
      for d in $NIX_STORE/*-cargo-crate-shlex-1.3.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: shlex-1.3.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/shlex-1.3.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/shlex-1.3.0/"
      cat > "/tmp/vendor/shlex-1.3.0/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-slab-0.4.12; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: slab-0.4.12" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/slab-0.4.12"
      cp -r "$CRATE_SRC/." "/tmp/vendor/slab-0.4.12/"
      cat > "/tmp/vendor/slab-0.4.12/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"}
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
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-0.2.121/"
      cat > "/tmp/vendor/wasm-bindgen-0.2.121/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"49ace1d07c165b0864824eee619580c4689389afa9dc9ed3a4c75040d82e6790"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-macro-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-macro-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-macro-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-macro-0.2.121/"
      cat > "/tmp/vendor/wasm-bindgen-macro-0.2.121/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"8e68e6f4afd367a562002c05637acb8578ff2dea1943df76afb9e83d177c8578"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-macro-support-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-macro-support-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-macro-support-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-macro-support-0.2.121/"
      cat > "/tmp/vendor/wasm-bindgen-macro-support-0.2.121/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"d95a9ec35c64b2a7cb35d3fead40c4238d0940c86d107136999567a4703259f2"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-shared-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-shared-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-shared-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-shared-0.2.121/"
      cat > "/tmp/vendor/wasm-bindgen-shared-0.2.121/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"c4e0100b01e9f0d03189a92b96772a1fb998639d981193d7dbab487302513441"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-core-0.62.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-core-0.62.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-core-0.62.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-core-0.62.2/"
      cat > "/tmp/vendor/windows-core-0.62.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-implement-0.60.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-implement-0.60.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-implement-0.60.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-implement-0.60.2/"
      cat > "/tmp/vendor/windows-implement-0.60.2/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-interface-0.59.3; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-interface-0.59.3" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-interface-0.59.3"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-interface-0.59.3/"
      cat > "/tmp/vendor/windows-interface-0.59.3/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358"}
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
      for d in $NIX_STORE/*-cargo-crate-windows-result-0.4.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-result-0.4.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-result-0.4.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-result-0.4.1/"
      cat > "/tmp/vendor/windows-result-0.4.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-strings-0.5.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-strings-0.5.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-strings-0.5.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-strings-0.5.1/"
      cat > "/tmp/vendor/windows-strings-0.5.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091"}
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
      for d in $NIX_STORE/*-cargo-crate-yansi-1.0.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: yansi-1.0.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/yansi-1.0.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/yansi-1.0.1/"
      cat > "/tmp/vendor/yansi-1.0.1/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"cfe53a6657fd280eaa890a3bc59152892ffa3e30101319d168b781ed6529b049"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zerocopy-0.8.48; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zerocopy-0.8.48" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zerocopy-0.8.48"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zerocopy-0.8.48/"
      cat > "/tmp/vendor/zerocopy-0.8.48/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"eed437bf9d6692032087e337407a86f04cd8d6a16a37199ed57949d415bd68e9"}
CHECKSUMEOF
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zerocopy-derive-0.8.48; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zerocopy-derive-0.8.48" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zerocopy-derive-0.8.48"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zerocopy-derive-0.8.48/"
      cat > "/tmp/vendor/zerocopy-derive-0.8.48/.cargo-checksum.json" <<CHECKSUMEOF
{"files":{},"package":"70e3cd084b1788766f53af483dd21f93881ff30d7320490ec3ef7526d203bad4"}
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

      cargo build --release --locked --offline -p clanker-message --target x86_64-unknown-linux-musl

      mkdir -p $out/lib $out/share/clankers
      cp $CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/libclanker_message.rlib $out/lib/
      cat > $out/share/clankers/clanker-message-build-receipt.txt << RECEIPT
package=clanker-message
cargo_command=cargo build --release --locked --offline -p clanker-message --target x86_64-unknown-linux-musl
source_archive_sha256=9de8eab60bf4762904727bb1f0886283f28e4b2ce30966812e9561030b17f3b1
registry_crate_count=57
artifact=$out/lib/libclanker_message.rlib
RECEIPT
      test -s $out/lib/libclanker_message.rlib
    "%,
  ],
  inputs = [toolchain, musl, rust, src, crate_0, crate_1, crate_2, crate_3, crate_4, crate_5, crate_6, crate_7, crate_8, crate_9, crate_10, crate_11, crate_12, crate_13, crate_14, crate_15, crate_16, crate_17, crate_18, crate_19, crate_20, crate_21, crate_22, crate_23, crate_24, crate_25, crate_26, crate_27, crate_28, crate_29, crate_30, crate_31, crate_32, crate_33, crate_34, crate_35, crate_36, crate_37, crate_38, crate_39, crate_40, crate_41, crate_42, crate_43, crate_44, crate_45, crate_46, crate_47, crate_48, crate_49, crate_50, crate_51, crate_52, crate_53, crate_54, crate_55, crate_56],
} | crunch.Derivation
