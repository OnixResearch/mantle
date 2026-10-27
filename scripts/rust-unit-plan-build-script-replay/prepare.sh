#!/usr/bin/env bash
# Prepare immutable real Cargo fixtures and a Nix-signed input cache; never reuse a prior proof bundle.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
B=${RCBS_B:-$R/tmp/rust-efficiency-20261011/rcbs-unit}
TARGET=$R/targets/rust-efficiency-20261011
MANTLE=$TARGET/release/mantle
STATIC=$TARGET/static/x86_64-unknown-linux-gnu/debug
TOOLCHAIN=/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08
COMPILER=${RCBS_COMPILER:?supply the admitted cargo-unit-compiler store root}
LINKER=/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0
PKG_CONFIG=/nix/store/d0immrndpxnhx32vcfbxngmnf7hmqyc3-pkg-config-wrapper-0.29.2
VENDOR=/nix/store/l5bfq4vwqh1sifhvkajjf0iimh7yg6b3-build-script-vendor
GRAPHS=/nix/store/d43yiwfqg9ls684sjcazn42vjn937m71-bs-graphs
BZ=/nix/store/dqgqqczf6pzb59pg7r13p80aa3xvnwr7-bz-sys
LINKS=/nix/store/6rhlil1lnr0h5zspa7ivdm65bhnsak0m-links-meta
BUSYBOX=/nix/store/h75l4bwd606k94l22ajqdb98vfar5x66-busybox-static-x86_64-unknown-linux-musl-1.37.0
[[ ! -e $B ]] || { echo "existing scratch is not a clean proof: $B" >&2; exit 1; }
for f in "$MANTLE" "$STATIC/mantle-cargo-unit-helper" "$STATIC/mantle-cargo-unit-producer"; do test -x "$f"; done
for d in "$TOOLCHAIN" "$COMPILER" "$LINKER" "$PKG_CONFIG" "$VENDOR" "$GRAPHS" "$BZ" "$LINKS" "$BUSYBOX"; do test -e "$d"; done
"$PKG_CONFIG/bin/pkg-config" --version >/dev/null
mkdir -p "$B/stage" "$B/keys" "$B/ncl" "$B/runs" "$B/offline"
add_binary() { local d; d=$(mktemp -d "$B/stage/binary.XXXXXXXX"); cp "$1" "$d/$2"; nix-store --add "$d/$2"; }
PRODUCER=$(add_binary "$STATIC/mantle-cargo-unit-producer" mantle-cargo-unit-producer)
HELPER=$(add_binary "$STATIC/mantle-cargo-unit-helper" mantle-cargo-unit-helper)
PLANNER=$(add_binary "$MANTLE" mantle)
for mode in nonzero host network; do
  cp -r "$LINKS" "$B/stage/$mode"
  chmod -R u+w "$B/stage/$mode"
done
cat > "$B/stage/nonzero/foo-sys/build.rs" <<'RS'
fn main() { std::process::exit(42); }
RS
cat > "$B/stage/host/foo-sys/build.rs" <<'RS'
fn main() {
    assert!(!std::path::Path::new("/etc/hostname").exists(), "undeclared-host-file-visible");
    std::fs::write(format!("{}/host-denied", std::env::var("OUT_DIR").unwrap()), "host-denied").unwrap();
    println!("cargo::metadata=include=/declared/foo/include");
    println!("cargo::rustc-cfg=foo_sys_built");
}
RS
[[ ${RCBS_NETWORK_PORT:-} =~ ^[0-9]+$ ]] || { echo 'missing controlled network probe' >&2; exit 1; }
cat > "$B/stage/network/foo-sys/build.rs" <<RS
fn main() {
    let probe = "127.0.0.1:${RCBS_NETWORK_PORT}".parse().unwrap();
    assert!(std::net::TcpStream::connect_timeout(&probe, std::time::Duration::from_secs(1)).is_err(), "network-reachable");
    std::fs::write(format!("{}/network-denied", std::env::var("OUT_DIR").unwrap()), "network-denied").unwrap();
    println!("cargo::metadata=include=/declared/foo/include");
    println!("cargo::rustc-cfg=foo_sys_built");
}
RS
test -e /etc/hostname || { echo 'host sentinel /etc/hostname absent' >&2; exit 1; }
ROOT_NONZERO=$(nix-store --add "$B/stage/nonzero")
ROOT_HOST=$(nix-store --add "$B/stage/host")
ROOT_NETWORK=$(nix-store --add "$B/stage/network")
# No historical binary or result is copied; immutable historical fixture *sources* are admitted by signed cache.
nix key generate-secret --key-name rcbs-rust-efficiency-20261011 > "$B/keys/secret"
nix key convert-secret-to-public < "$B/keys/secret" > "$B/keys/public"
ROOTS=("$PRODUCER" "$HELPER" "$PLANNER" "$TOOLCHAIN" "$COMPILER" "$LINKER" "$PKG_CONFIG" "$BUSYBOX" "$VENDOR" "$GRAPHS" "$BZ" "$LINKS" "$ROOT_NONZERO" "$ROOT_HOST" "$ROOT_NETWORK")
nix copy --offline --to "file://$B/input-cache?compression=none&secret-key=$B/keys/secret" "${ROOTS[@]}"
jq -n --arg producer "$PRODUCER" --arg helper "$HELPER" --arg planner "$PLANNER" \
  --arg toolchain "$TOOLCHAIN" --arg compiler "$COMPILER" --arg linker "$LINKER" --arg pkg_config "$PKG_CONFIG" \
  --arg vendor "$VENDOR" --arg graphs "$GRAPHS" --arg bz "$BZ" --arg links "$LINKS" \
  --arg nonzero "$ROOT_NONZERO" --arg host "$ROOT_HOST" --arg network "$ROOT_NETWORK" \
  --arg public_key "$(cat "$B/keys/public")" --arg busybox "$BUSYBOX" '$ARGS.named' > "$B/paths.json"
sha256sum "$MANTLE" "$STATIC/mantle-cargo-unit-helper" "$STATIC/mantle-cargo-unit-producer" > "$B/binary-sha256.txt"
[[ $(find "$B/input-cache" -maxdepth 1 -name '*.narinfo' -type f | wc -l) -ge ${#ROOTS[@]} ]] || { echo 'incomplete signed cache' >&2; exit 1; }
printf 'signed input cache: %s; immutable roots: %s\n' "$B/input-cache" "${#ROOTS[@]}"
