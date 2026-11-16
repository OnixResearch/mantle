#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH
DEV="$ROOT/signed-stagex-adoption-dev-20261116"
SMOKE="$ROOT/tools-source-planning-smoke-20261116"
CAPTURED="$ROOT/.prepared-cohort-d911681e-signed-stagex-adoption.source-built-fixed-point-staging-3900186/inputs/mantle-source"
VENDOR="$ROOT/.prepared-cohort-d911681e-signed-stagex-adoption.source-built-fixed-point-staging-3900186/inputs/vendor-deps"
MANTLE="$ROOT/bin/mantle-d911681e-signed-stagex-adoption"
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$SMOKE"
mkdir "$SMOKE"
exec > "$SMOKE/verification.log" 2>&1
trap 'status=$?; printf "tools_source_verification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$SMOKE/verification.exit"; exit "$status"' EXIT
set -x
test "$("$B3" --no-names "$MANTLE")" = 2747154bd1e3b5b54c0d6387ba79eb7ca184751d2b244a0da89c445e3e539fad
test ! -e "$CAPTURED/scripts"
test -d "$CAPTURED/tools"
cp -a --reflink=auto "$CAPTURED" "$SMOKE/source"
chmod u+w "$SMOKE/source" "$SMOKE/source/tools"
install -m 644 "$DEV/Cargo.toml" "$SMOKE/source/Cargo.toml"
install -m 644 "$DEV/tools/adopt_source_built_stagex.rs" "$SMOKE/source/tools/adopt_source_built_stagex.rs"
install -m 644 "$DEV/tools/check_native_row_receipts.rs" "$SMOKE/source/tools/check_native_row_receipts.rs"
cp -a --reflink=auto "$VENDOR" "$SMOKE/source/vendor-deps"
mkdir "$SMOKE/tmp"
export SMOKE MANTLE
export CARGO_TARGET_DIR="$ROOT/canonical-target-869a461d-request-workspace" TMPDIR="$SMOKE/tmp" XDG_CACHE_HOME="$ROOT/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
cd "$DEV"
/run/current-system/sw/bin/nix develop --profile "$ROOT/tooling-profile-signed-stagex-adoption" --builders '' --no-write-lock-file -c bash -c '
set -euo pipefail
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
cargo fmt --check -p mantle
cargo clippy --release --locked --offline -p mantle --bin adopt-source-built-stagex --bin check-native-row-receipts --no-deps -- -D warnings
cargo build --release --locked --offline -p mantle --bin adopt-source-built-stagex --bin check-native-row-receipts
"$CARGO_TARGET_DIR/release/adopt-source-built-stagex" --help
"$CARGO_TARGET_DIR/release/check-native-row-receipts" --contract
rustc -Vv
timeout 300 "$MANTLE" --json --state-dir "$SMOKE/state" rust-plan --root "$SMOKE/source" --rustc "$(command -v rustc)" --no-cargo-oracle > "$SMOKE/native-plan.json"
test "$(stat -c %s "$SMOKE/native-plan.json")" -le 67108864
jq -e ".native_package_target_planning.ready == true and ([.native_package_target_planning.packages[] | select(.name == \"mantle\") | .targets[] | select(.name == \"adopt-source-built-stagex\" or .name == \"check-native-row-receipts\")] | length == 2)" "$SMOKE/native-plan.json"
'
printf 'tools_source_verification_complete=1\n'
