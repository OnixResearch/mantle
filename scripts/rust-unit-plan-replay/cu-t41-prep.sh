#!/usr/bin/env bash
# T4.1/T4.2 stage A: add immutable inputs to /nix/store, write producer configs, build a Nix-signed input cache.
# Run from the isolated Mantle repository with the pinned Nix/Cargo dev shell.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
F=$R/tmp/cu-fixture
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
MANTLE=${CU_MANTLE:-$R/targets/rust-efficiency-20261011/release/mantle}
STATIC=${CU_STATIC:-$R/targets/rust-efficiency-20261011/static/x86_64-unknown-linux-gnu/debug}
APP_CONSUMER=${CU_APP_CONSUMER:-$R/targets/rust-efficiency-20261011/app-consumer}
TOOLCHAIN=/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08
COMPILER=${CU_COMPILER:?supply the admitted cargo-unit-compiler store root}
LINKER=/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0
MUSL=/nix/store/0nl9w8qhb0kvix80vpa191a59mkhw1q9-musl-x86_64-unknown-linux-musl-1.2.5
W0=/nix/store/2cc8iz424blki3kgz09ajdndy0n6pjlc-baseline-workspace

test ! -e "$B" || { echo "refusing to overwrite existing proof bundle $B" >&2; exit 1; }
mkdir -p "$B/stage/bin" "$B/stage/planner" "$B/stage/app-edit" "$B/stage/outside-edit" "$B/keys"
cd "$WT"
echo "branch_head=$(git rev-parse HEAD)"
sha256sum "$MANTLE" "$STATIC/mantle-cargo-unit-producer" "$STATIC/mantle-cargo-unit-helper" "$APP_CONSUMER"

add_file() { # add_file SRC NAME -> store path
  local dir
  dir=$(mktemp -d "$B/stage/add.XXXXXX")
  cp "$1" "$dir/$2"
  nix-store --add "$dir/$2"
}
PRODUCER=$(add_file "$STATIC/mantle-cargo-unit-producer" mantle-cargo-unit-producer)
HELPER=$(add_file "$STATIC/mantle-cargo-unit-helper" mantle-cargo-unit-helper)
PLANNER=$(add_file "$MANTLE" mantle)
APPC=$(add_file "$APP_CONSUMER" app-consumer)

# Workspaces: W0 is the immutable baseline; W1 edits only the app package; W2 adds a file outside every package.
[[ $(nix-store --add "$F/baseline-workspace") == "$W0" ]] || { echo "baseline workspace does not reproduce $W0" >&2; exit 1; }
cp -r "$F/baseline-workspace" "$B/stage/app-edit/baseline-workspace"
chmod -R u+w "$B/stage/app-edit/baseline-workspace"
printf '%s\n' 'fn main() { assert_eq!(unit_baseline_dep::answer(), 42); println!("unit-plan-app-edit"); }' \
  > "$B/stage/app-edit/baseline-workspace/app/src/main.rs"
W1=$(nix-store --add "$B/stage/app-edit/baseline-workspace")
cp -r "$F/baseline-workspace" "$B/stage/outside-edit/baseline-workspace"
chmod -R u+w "$B/stage/outside-edit/baseline-workspace"
cp "$F/variants/README.txt" "$B/stage/outside-edit/baseline-workspace/README.txt"
W2=$(nix-store --add "$B/stage/outside-edit/baseline-workspace")

config_for() { # config_for WORKSPACE TAG -> store path of producer-config.json
  local dir="$B/stage/config-$2"
  mkdir -p "$dir"
  jq -cn --arg workspace "$1" --arg planner "$PLANNER" --arg toolchain "$TOOLCHAIN" --arg compiler "$COMPILER" --arg linker "$LINKER" --arg helper "$HELPER" '{
    workspace: $workspace, vendor: null, planner: $planner,
    cargo: ($toolchain + "/bin/cargo"), rustc: ($compiler + "/bin/rustc"), linker: ($linker + "/bin/cc"), runtime_inputs: [$compiler],
    toolchain: $toolchain, helper: $helper, store_prefix: "/nix/store", system: "x86_64-linux",
    host_triple: "x86_64-unknown-linux-gnu", target_triple: "x86_64-unknown-linux-gnu", profile: "dev"}' > "$dir/producer-config.json"
  nix-store --add "$dir/producer-config.json"
}
CFG0=$(config_for "$W0" baseline)
CFG1=$(config_for "$W1" app-edit)
CFG2=$(config_for "$W2" outside-edit)

SHIM=$(nix-build --no-out-link "$WT/scripts/rust-unit-plan-replay/shim.nix")
nix key generate-secret --key-name cargo-unit-rust-efficiency-1 > "$B/keys/secret"
nix key convert-secret-to-public < "$B/keys/secret" > "$B/keys/public"
ROOTS=("$PRODUCER" "$HELPER" "$PLANNER" "$TOOLCHAIN" "$COMPILER" "$LINKER" "$MUSL" "$SHIM" "$W0" "$W1" "$W2" "$CFG0" "$CFG1" "$CFG2" "$APPC")
nix copy --offline --to "file://$B/input-cache?compression=none&secret-key=$B/keys/secret" "${ROOTS[@]}"
echo "input_cache_narinfos=$(find "$B/input-cache" -maxdepth 1 -name '*.narinfo' -type f | wc -l) input_cache_bytes=$(du -sb "$B/input-cache" | cut -f1)"

jq -n --arg producer "$PRODUCER" --arg helper "$HELPER" --arg planner "$PLANNER" --arg app_consumer "$APPC" \
  --arg toolchain "$TOOLCHAIN" --arg compiler "$COMPILER" --arg linker "$LINKER" --arg musl "$MUSL" --arg shim "$SHIM" \
  --arg w0 "$W0" --arg w1 "$W1" --arg w2 "$W2" \
  --arg cfg0 "$CFG0" --arg cfg1 "$CFG1" --arg cfg2 "$CFG2" --arg public_key "$(cat "$B/keys/public")" \
  '$ARGS.named' | tee "$B/paths.json"
