#!/usr/bin/env bash
set -euo pipefail
RUN='/home/brittonr/sbfp-20261113-berdKH'
SOURCE='/home/brittonr/sbfp-20261113-berdKH/primary-source-paths-dev-20261117'
LOG="$RUN/primary-source-paths-dev-20261117-v2.log"
test ! -e "$LOG"
exec > "$LOG" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "primary_source_developer_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/primary-source-paths-dev-20261117-v2.exit"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR='/home/brittonr/sbfp-20261113-berdKH/canonical-target-869a461d-request-workspace'
export TMPDIR="$RUN/canonical-tmp-869a461d-request-workspace" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
'/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' src/rust_plan.rs
/run/current-system/sw/bin/nix develop --profile "$RUN/tooling-profile-primary-source-paths-dev" --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; cargo fmt --check -p mantle; cargo test --release --locked --offline -p mantle --bin mantle rust_plan::tests::source_only_topology_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --no-deps -- -D warnings; cargo build --release --locked --offline -p mantle --bin mantle'
"$CARGO_TARGET_DIR/release/mantle" --version
'/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' src/rust_plan.rs
finished=1
printf 'primary_source_developer_complete=1\n'
