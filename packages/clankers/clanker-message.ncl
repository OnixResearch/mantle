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
  hash = "sha256-nejqtgv0dikEcnux8Ihig/KOSyzjCWaBLpVhAwsX87E=",
  name = "clanker-message-src",
} in

let crate_0 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/android_system_properties/android_system_properties-0.1.5.crate",
  hash = "sha256-gZ5yGdvUEEOsJ5sZgw8u/IlxVkkNf9bqkWcgEX7mYxE=",
  name = "cargo-crate-android_system_properties-0.1.5",
} in
let crate_1 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/autocfg/autocfg-1.5.0.crate",
  hash = "sha256-wIYG+MPL9M5uyOKPsAFKLAhnCP6VTqqIU4SmFlFy5+g=",
  name = "cargo-crate-autocfg-1.5.0",
} in
let crate_2 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/bumpalo/bumpalo-3.20.2.crate",
  hash = "sha256-XSB4mGj0sBsvLK7J9cTgITtB4+VwKlAVfWma4xztL8s=",
  name = "cargo-crate-bumpalo-3.20.2",
} in
let crate_3 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/cc/cc-1.2.62.crate",
  hash = "sha256-odzoWfCDKn0IjE8RGYiKuU70tdZ5XRzgWvt/4VnXn5g=",
  name = "cargo-crate-cc-1.2.62",
} in
let crate_4 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/cfg-if/cfg-if-1.0.4.crate",
  hash = "sha256-kzD4sv8T80VAtE6UbvNREYJXJ7ONMyhu+YYUJhUSGAE=",
  name = "cargo-crate-cfg-if-1.0.4",
} in
let crate_5 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/chrono/chrono-0.4.44.crate",
  hash = "sha256-xnMHWi4OX0od3ifOne4epFWMf/5kj1dkOKIModKsxLA=",
  name = "cargo-crate-chrono-0.4.44",
} in
let crate_6 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/core-foundation-sys/core-foundation-sys-0.8.7.crate",
  hash = "sha256-dzZIuU0OXWIPZPKAd3RFdA5h/nAQJQh+yLV/RceRiIs=",
  name = "cargo-crate-core-foundation-sys-0.8.7",
} in
let crate_7 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/diff/diff-0.1.13.crate",
  hash = "sha256-ViVJhndeMjP/qcTX0/qvbTaiwJ0wsgaH6fiLyLr8Fsg=",
  name = "cargo-crate-diff-0.1.13",
} in
let crate_8 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/find-msvc-tools/find-msvc-tools-0.1.9.crate",
  hash = "sha256-W668B3QVH5BaGizEGYkwCx5vuymv8M7/oQZP3TCI1YI=",
  name = "cargo-crate-find-msvc-tools-0.1.9",
} in
let crate_9 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-core/futures-core-0.3.32.crate",
  hash = "sha256-fjRQgVJy71jOxtVkQj9udV4lN5shewvGiOKVuiTfax0=",
  name = "cargo-crate-futures-core-0.3.32",
} in
let crate_10 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-task/futures-task-0.3.32.crate",
  hash = "sha256-A3cRs9WcMwBNOFb73IO5nU/zeiR2j6G+nONTihzeQ5M=",
  name = "cargo-crate-futures-task-0.3.32",
} in
let crate_11 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/futures-util/futures-util-0.3.32.crate",
  hash = "sha256-OJykEpbmGQtIBT3gMh0Cp38y+KXSRh3Th2LAWTgFxtY=",
  name = "cargo-crate-futures-util-0.3.32",
} in
let crate_12 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/getrandom/getrandom-0.3.4.crate",
  hash = "sha256-iZ3vXDfE/XsmZGSMKBIOzsE45NOVtFnlyjT5zOLdd/0=",
  name = "cargo-crate-getrandom-0.3.4",
} in
let crate_13 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/hex/hex-0.4.3.crate",
  hash = "sha256-fyQlSqmlS1yFjq7i9bzNtGqvDkhqWV7V/Y+GulUjKnA=",
  name = "cargo-crate-hex-0.4.3",
} in
let crate_14 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/iana-time-zone/iana-time-zone-0.1.65.crate",
  hash = "sha256-4xvJrZlLoA5ECoqlye8Oxn1cteXLDMf4t0SjWzicxHA=",
  name = "cargo-crate-iana-time-zone-0.1.65",
} in
let crate_15 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/iana-time-zone-haiku/iana-time-zone-haiku-0.1.2.crate",
  hash = "sha256-8xgnogb1avMuWQulbV0tCF9VhQgZJZN0PxayMGSVJp8=",
  name = "cargo-crate-iana-time-zone-haiku-0.1.2",
} in
let crate_16 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/itoa/itoa-1.0.18.crate",
  hash = "sha256-j0KmDL35qX9dIwXwiofcTgkwjRJ20oyGnGhNd3doVoI=",
  name = "cargo-crate-itoa-1.0.18",
} in
let crate_17 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/js-sys/js-sys-0.3.98.crate",
  hash = "sha256-Z99xEmE/i/2RUAE6AxThlvSADTIBrnQkidmZ2y+Xnwg=",
  name = "cargo-crate-js-sys-0.3.98",
} in
let crate_18 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/libc/libc-0.2.186.crate",
  hash = "sha256-aKuRAX/hbGIkhoQOTIPJo3r+/5eL0jm1KT1h7OWH3mY=",
  name = "cargo-crate-libc-0.2.186",
} in
let crate_19 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/log/log-0.4.29.crate",
  hash = "sha256-XlAy4kAZBFx2LTwPKPW2uLvzhWOmWQg4m/eXh1iSCJc=",
  name = "cargo-crate-log-0.4.29",
} in
let crate_20 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/memchr/memchr-2.8.0.crate",
  hash = "sha256-+MpY9EfwbtF9X8QEPOGxDdIF4GD7POW5ebjtjln/P3k=",
  name = "cargo-crate-memchr-2.8.0",
} in
let crate_21 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/num-traits/num-traits-0.2.19.crate",
  hash = "sha256-Bx38BiaQ6QtzTAsic85yrQ/6lfDHRZa8JQ3P2WAmKEE=",
  name = "cargo-crate-num-traits-0.2.19",
} in
let crate_22 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/once_cell/once_cell-1.21.4.crate",
  hash = "sha256-n3w+S+sz+F1Frj46F5IYVwbI4W0EMjjFkzMcx80xO1A=",
  name = "cargo-crate-once_cell-1.21.4",
} in
let crate_23 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/pin-project-lite/pin-project-lite-0.2.17.crate",
  hash = "sha256-qJMi356+HBV41onJIxjgcJZ9EEK1Eq++SVGHI/Tm1c0=",
  name = "cargo-crate-pin-project-lite-0.2.17",
} in
let crate_24 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/ppv-lite86/ppv-lite86-0.2.21.crate",
  hash = "sha256-herjxO0vUNz+cmQ9pL78MN6ttFiptZDXIM3i8rHpfak=",
  name = "cargo-crate-ppv-lite86-0.2.21",
} in
let crate_25 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/pretty_assertions/pretty_assertions-1.4.1.crate",
  hash = "sha256-OuEw4vJx+8KsOkD7HQcYCDnNu+RDx6J+HjwTxcrAEW0=",
  name = "cargo-crate-pretty_assertions-1.4.1",
} in
let crate_26 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/proc-macro2/proc-macro2-1.0.106.crate",
  hash = "sha256-j9APC7LpDYHRBEwrMmF/aPy5+ju3ZAwj6cdI5T+zCTQ=",
  name = "cargo-crate-proc-macro2-1.0.106",
} in
let crate_27 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/quote/quote-1.0.45.crate",
  hash = "sha256-QfJhmWYFBok4LStE9mT0vFk+EpeFo21u43bd83JZuSQ=",
  name = "cargo-crate-quote-1.0.45",
} in
let crate_28 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/r-efi/r-efi-5.3.0.crate",
  hash = "sha256-ac2zTBWM6yiN8R4YtL053plPZlfYOEe9/9vX80Z1Sw8=",
  name = "cargo-crate-r-efi-5.3.0",
} in
let crate_29 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand/rand-0.9.4.crate",
  hash = "sha256-RMWvBrsbfTIW2RkyrtUmUWS/OE3InNa6Bc9Zo19fduo=",
  name = "cargo-crate-rand-0.9.4",
} in
let crate_30 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand_chacha/rand_chacha-0.9.0.crate",
  hash = "sha256-0wIrXx32Dybh/93Wxm6KoV3jgq5js6DBv8Dk0+PzJcs=",
  name = "cargo-crate-rand_chacha-0.9.0",
} in
let crate_31 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rand_core/rand_core-0.9.5.crate",
  hash = "sha256-dq/IJt4UI45ujDdN3MH6GeN0/Y3ZhrDSrw0CN3Jh2Dw=",
  name = "cargo-crate-rand_core-0.9.5",
} in
let crate_32 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/rustversion/rustversion-1.0.22.crate",
  hash = "sha256-s5ze8PqAD8RFJchMy1SgKZYaghX5YZdTY1qcDSU41G0=",
  name = "cargo-crate-rustversion-1.0.22",
} in
let crate_33 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde/serde-1.0.228.crate",
  hash = "sha256-mo6U6n83i9Msu9NxmKSpFDYYDFu0ckEeSLXsLiEkrp4=",
  name = "cargo-crate-serde-1.0.228",
} in
let crate_34 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_core/serde_core-1.0.228.crate",
  hash = "sha256-QdOFx9TKWOWfxzKvJcOYO2eshSwaJQAK/hF13kWLZ60=",
  name = "cargo-crate-serde_core-1.0.228",
} in
let crate_35 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_derive/serde_derive-1.0.228.crate",
  hash = "sha256-1UDyINMYcXPaIg+IWrZmCDZ7ZXTpJQEak1Pkut2pHXk=",
  name = "cargo-crate-serde_derive-1.0.228",
} in
let crate_36 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/serde_json/serde_json-1.0.149.crate",
  hash = "sha256-g/wDlHPFWVrOhg2MT6+iIP9HSz/Gv9tCkzJ/GjfpTYY=",
  name = "cargo-crate-serde_json-1.0.149",
} in
let crate_37 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/shlex/shlex-1.3.0.crate",
  hash = "sha256-D9ov8NCEAZuk18bzccldj9dc41JMPLj7ZTowI/YyPmQ=",
  name = "cargo-crate-shlex-1.3.0",
} in
let crate_38 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/slab/slab-0.4.12.crate",
  hash = "sha256-DHkN4jEk+atEVE16wF1gRArcWGR5zlAcHW19o82MnPU=",
  name = "cargo-crate-slab-0.4.12",
} in
let crate_39 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/syn/syn-2.0.117.crate",
  hash = "sha256-5mW4gD57HSpyf0AjRWu7vnTaZwmcWFJYrwrZxQE7m5k=",
  name = "cargo-crate-syn-2.0.117",
} in
let crate_40 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/unicode-ident/unicode-ident-1.0.24.crate",
  hash = "sha256-5uQxPNX809rVyvoXlwLiskT3YJkfRTl9FNTr84JH2nU=",
  name = "cargo-crate-unicode-ident-1.0.24",
} in
let crate_41 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasip2/wasip2-1.0.3+wasi-0.2.9.crate",
  hash = "sha256-IAZGctsm183InHeYxIoP36yCE0NKEYbl7yn9VgriI9Y=",
  name = "cargo-crate-wasip2-1.0.3-wasi-0.2.9",
} in
let crate_42 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen/wasm-bindgen-0.2.121.crate",
  hash = "sha256-Sazh0HwWWwhkgk7uYZWAxGiTia+p3J7TpMdQQNguZ5A=",
  name = "cargo-crate-wasm-bindgen-0.2.121",
} in
let crate_43 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-macro/wasm-bindgen-macro-0.2.121.crate",
  hash = "sha256-jmjm9K/TZ6ViACwFY3rLhXj/LeoZQ992r7noPRd8hXg=",
  name = "cargo-crate-wasm-bindgen-macro-0.2.121",
} in
let crate_44 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-macro-support/wasm-bindgen-macro-support-0.2.121.crate",
  hash = "sha256-2Vqew1xksqfLNdP+rUDEI40JQMhtEHE2mZVnpHAyWfI=",
  name = "cargo-crate-wasm-bindgen-macro-support-0.2.121",
} in
let crate_45 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wasm-bindgen-shared/wasm-bindgen-shared-0.2.121.crate",
  hash = "sha256-xOAQCwHp8NAxiakrlncqH7mYY52YEZPX26tIcwJRNEE=",
  name = "cargo-crate-wasm-bindgen-shared-0.2.121",
} in
let crate_46 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-core/windows-core-0.62.2.crate",
  hash = "sha256-uOg6FNNNBiO1Hc6VgRmTAqIhhjGWod3nGnZjpMK+nes=",
  name = "cargo-crate-windows-core-0.62.2",
} in
let crate_47 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-implement/windows-implement-0.60.2.crate",
  hash = "sha256-BT4uBAq1e53JUbcsJkhg236zsCALo0W05MOxT2eFXd8=",
  name = "cargo-crate-windows-implement-0.60.2",
} in
let crate_48 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-interface/windows-interface-0.59.3.crate",
  hash = "sha256-PzFsSiVwuia77HIgMsQJnYyLwJXvzNwVaIcIYjNn41g=",
  name = "cargo-crate-windows-interface-0.59.3",
} in
let crate_49 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-link/windows-link-0.2.1.crate",
  hash = "sha256-8IBSIuV/dSHWpi42+pFjvIkazUIvlx3v6X1k5w0KT+U=",
  name = "cargo-crate-windows-link-0.2.1",
} in
let crate_50 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-result/windows-result-0.4.1.crate",
  hash = "sha256-d4H6ier2CFCsPS2nr45SQqXqeNGhHEm/KRC7WnOFPrU=",
  name = "cargo-crate-windows-result-0.4.1",
} in
let crate_51 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/windows-strings/windows-strings-0.5.1.crate",
  hash = "sha256-eDfQj2nHfPawdolURTjgF8G/z1fjS0wP9Y5sLNOzcJE=",
  name = "cargo-crate-windows-strings-0.5.1",
} in
let crate_52 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/wit-bindgen/wit-bindgen-0.57.1.crate",
  hash = "sha256-Hr+UToenwlMjOtZ2bgguPNcUtdA4EqzCTDGPVJYUU24=",
  name = "cargo-crate-wit-bindgen-0.57.1",
} in
let crate_53 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/yansi/yansi-1.0.1.crate",
  hash = "sha256-z+U6Zlf9KA6qiQo7xZFSiS/6PjAQExnRaLeB7WUpsEk=",
  name = "cargo-crate-yansi-1.0.1",
} in
let crate_54 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zerocopy/zerocopy-0.8.48.crate",
  hash = "sha256-7tQ3v51mkgMgh+M3QHqG8EzY1qFqNxme1XlJ1BW9aOk=",
  name = "cargo-crate-zerocopy-0.8.48",
} in
let crate_55 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zerocopy-derive/zerocopy-derive-0.8.48.crate",
  hash = "sha256-cOPNCEsXiHZvU69IPdIfk4gf8w1zIEkOw+91JtIDutQ=",
  name = "cargo-crate-zerocopy-derive-0.8.48",
} in
let crate_56 = crunch.fetchTarball {
  url = "https://static.crates.io/crates/zmij/zmij-1.0.21.crate",
  hash = "sha256-uISO5n7Miu268+QSIhev+JJjkjG+/GobWNKf/0wsq6o=",
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
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-autocfg-1.5.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: autocfg-1.5.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/autocfg-1.5.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/autocfg-1.5.0/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-bumpalo-3.20.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: bumpalo-3.20.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/bumpalo-3.20.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/bumpalo-3.20.2/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-cc-1.2.62; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: cc-1.2.62" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/cc-1.2.62"
      cp -r "$CRATE_SRC/." "/tmp/vendor/cc-1.2.62/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-cfg-if-1.0.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: cfg-if-1.0.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/cfg-if-1.0.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/cfg-if-1.0.4/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-chrono-0.4.44; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: chrono-0.4.44" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/chrono-0.4.44"
      cp -r "$CRATE_SRC/." "/tmp/vendor/chrono-0.4.44/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-core-foundation-sys-0.8.7; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: core-foundation-sys-0.8.7" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/core-foundation-sys-0.8.7"
      cp -r "$CRATE_SRC/." "/tmp/vendor/core-foundation-sys-0.8.7/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-diff-0.1.13; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: diff-0.1.13" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/diff-0.1.13"
      cp -r "$CRATE_SRC/." "/tmp/vendor/diff-0.1.13/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-find-msvc-tools-0.1.9; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: find-msvc-tools-0.1.9" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/find-msvc-tools-0.1.9"
      cp -r "$CRATE_SRC/." "/tmp/vendor/find-msvc-tools-0.1.9/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-core-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-core-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-core-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-core-0.3.32/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-task-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-task-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-task-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-task-0.3.32/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-futures-util-0.3.32; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: futures-util-0.3.32" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/futures-util-0.3.32"
      cp -r "$CRATE_SRC/." "/tmp/vendor/futures-util-0.3.32/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-getrandom-0.3.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: getrandom-0.3.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/getrandom-0.3.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/getrandom-0.3.4/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-hex-0.4.3; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: hex-0.4.3" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/hex-0.4.3"
      cp -r "$CRATE_SRC/." "/tmp/vendor/hex-0.4.3/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-iana-time-zone-0.1.65; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: iana-time-zone-0.1.65" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/iana-time-zone-0.1.65"
      cp -r "$CRATE_SRC/." "/tmp/vendor/iana-time-zone-0.1.65/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-iana-time-zone-haiku-0.1.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: iana-time-zone-haiku-0.1.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/iana-time-zone-haiku-0.1.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/iana-time-zone-haiku-0.1.2/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-itoa-1.0.18; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: itoa-1.0.18" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/itoa-1.0.18"
      cp -r "$CRATE_SRC/." "/tmp/vendor/itoa-1.0.18/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-js-sys-0.3.98; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: js-sys-0.3.98" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/js-sys-0.3.98"
      cp -r "$CRATE_SRC/." "/tmp/vendor/js-sys-0.3.98/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-libc-0.2.186; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: libc-0.2.186" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/libc-0.2.186"
      cp -r "$CRATE_SRC/." "/tmp/vendor/libc-0.2.186/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-log-0.4.29; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: log-0.4.29" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/log-0.4.29"
      cp -r "$CRATE_SRC/." "/tmp/vendor/log-0.4.29/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-memchr-2.8.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: memchr-2.8.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/memchr-2.8.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/memchr-2.8.0/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-num-traits-0.2.19; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: num-traits-0.2.19" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/num-traits-0.2.19"
      cp -r "$CRATE_SRC/." "/tmp/vendor/num-traits-0.2.19/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-once_cell-1.21.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: once_cell-1.21.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/once_cell-1.21.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/once_cell-1.21.4/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-pin-project-lite-0.2.17; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: pin-project-lite-0.2.17" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/pin-project-lite-0.2.17"
      cp -r "$CRATE_SRC/." "/tmp/vendor/pin-project-lite-0.2.17/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-ppv-lite86-0.2.21; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: ppv-lite86-0.2.21" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/ppv-lite86-0.2.21"
      cp -r "$CRATE_SRC/." "/tmp/vendor/ppv-lite86-0.2.21/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-pretty_assertions-1.4.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: pretty_assertions-1.4.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/pretty_assertions-1.4.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/pretty_assertions-1.4.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-proc-macro2-1.0.106; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: proc-macro2-1.0.106" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/proc-macro2-1.0.106"
      cp -r "$CRATE_SRC/." "/tmp/vendor/proc-macro2-1.0.106/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-quote-1.0.45; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: quote-1.0.45" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/quote-1.0.45"
      cp -r "$CRATE_SRC/." "/tmp/vendor/quote-1.0.45/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-r-efi-5.3.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: r-efi-5.3.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/r-efi-5.3.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/r-efi-5.3.0/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand-0.9.4; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand-0.9.4" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand-0.9.4"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand-0.9.4/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand_chacha-0.9.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand_chacha-0.9.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand_chacha-0.9.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand_chacha-0.9.0/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rand_core-0.9.5; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rand_core-0.9.5" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rand_core-0.9.5"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rand_core-0.9.5/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-rustversion-1.0.22; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: rustversion-1.0.22" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/rustversion-1.0.22"
      cp -r "$CRATE_SRC/." "/tmp/vendor/rustversion-1.0.22/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde-1.0.228/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_core-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_core-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_core-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_core-1.0.228/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_derive-1.0.228; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_derive-1.0.228" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_derive-1.0.228"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_derive-1.0.228/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-serde_json-1.0.149; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: serde_json-1.0.149" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/serde_json-1.0.149"
      cp -r "$CRATE_SRC/." "/tmp/vendor/serde_json-1.0.149/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-shlex-1.3.0; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: shlex-1.3.0" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/shlex-1.3.0"
      cp -r "$CRATE_SRC/." "/tmp/vendor/shlex-1.3.0/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-slab-0.4.12; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: slab-0.4.12" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/slab-0.4.12"
      cp -r "$CRATE_SRC/." "/tmp/vendor/slab-0.4.12/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-syn-2.0.117; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: syn-2.0.117" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/syn-2.0.117"
      cp -r "$CRATE_SRC/." "/tmp/vendor/syn-2.0.117/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-unicode-ident-1.0.24; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: unicode-ident-1.0.24" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/unicode-ident-1.0.24"
      cp -r "$CRATE_SRC/." "/tmp/vendor/unicode-ident-1.0.24/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasip2-1.0.3-wasi-0.2.9; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasip2-1.0.3+wasi-0.2.9" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasip2-1.0.3+wasi-0.2.9"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasip2-1.0.3+wasi-0.2.9/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-0.2.121/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-macro-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-macro-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-macro-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-macro-0.2.121/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-macro-support-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-macro-support-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-macro-support-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-macro-support-0.2.121/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wasm-bindgen-shared-0.2.121; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wasm-bindgen-shared-0.2.121" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wasm-bindgen-shared-0.2.121"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wasm-bindgen-shared-0.2.121/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-core-0.62.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-core-0.62.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-core-0.62.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-core-0.62.2/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-implement-0.60.2; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-implement-0.60.2" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-implement-0.60.2"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-implement-0.60.2/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-interface-0.59.3; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-interface-0.59.3" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-interface-0.59.3"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-interface-0.59.3/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-link-0.2.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-link-0.2.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-link-0.2.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-link-0.2.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-result-0.4.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-result-0.4.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-result-0.4.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-result-0.4.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-windows-strings-0.5.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: windows-strings-0.5.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/windows-strings-0.5.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/windows-strings-0.5.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-wit-bindgen-0.57.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: wit-bindgen-0.57.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/wit-bindgen-0.57.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/wit-bindgen-0.57.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-yansi-1.0.1; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: yansi-1.0.1" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/yansi-1.0.1"
      cp -r "$CRATE_SRC/." "/tmp/vendor/yansi-1.0.1/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zerocopy-0.8.48; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zerocopy-0.8.48" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zerocopy-0.8.48"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zerocopy-0.8.48/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zerocopy-derive-0.8.48; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zerocopy-derive-0.8.48" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zerocopy-derive-0.8.48"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zerocopy-derive-0.8.48/"
      CRATE_SRC=""
      for d in $NIX_STORE/*-cargo-crate-zmij-1.0.21; do
        if [ -f "$d/Cargo.toml" ]; then CRATE_SRC="$d"; break; fi
      done
      if [ -z "$CRATE_SRC" ]; then echo "ERROR: crate input missing: zmij-1.0.21" >&2; exit 1; fi
      mkdir -p "/tmp/vendor/zmij-1.0.21"
      cp -r "$CRATE_SRC/." "/tmp/vendor/zmij-1.0.21/"

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
