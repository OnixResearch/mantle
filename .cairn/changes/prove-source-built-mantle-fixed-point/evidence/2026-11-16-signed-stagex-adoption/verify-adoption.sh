#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE="$RUN/signed-stagex-adoption-dev-20261116"
ATTEMPT="${1:?verification attempt label required}"
LOG="$RUN/signed-stagex-adoption-focused-20261116-$ATTEMPT.log"
test ! -e "$LOG"
exec > "$LOG" 2>&1
trap 'status=$?; printf "signed_adoption_focused_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/signed-stagex-adoption-focused-20261116-$ATTEMPT.exit"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR="$RUN/canonical-target-869a461d-request-workspace"
export TMPDIR="$RUN/canonical-tmp-869a461d-request-workspace" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --profile "$RUN/tooling-profile-signed-stagex-adoption" --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash -c '
set -euo pipefail
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
rustc -Vv
cargo -V
cargo fmt --check -p mantle
cargo clippy --release --locked --offline -p mantle --bin adopt-source-built-stagex --bin check-native-row-receipts --no-deps -- -D warnings
cargo build --release --locked --offline -p mantle --bin adopt-source-built-stagex --bin check-native-row-receipts
"$CARGO_TARGET_DIR/release/adopt-source-built-stagex" --help
"$CARGO_TARGET_DIR/release/check-native-row-receipts" --contract
python3 scripts/qualify-native-rows.py --help
'
printf 'signed_adoption_focused_complete=1\n'
