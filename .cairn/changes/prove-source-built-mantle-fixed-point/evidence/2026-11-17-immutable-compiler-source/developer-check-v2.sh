#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/immutable-compiler-source-dev-20261117
exec > "$RUN/immutable-compiler-source-developer-v2-20261117.log" 2>&1
trap 'status=$?; printf "immutable_compiler_source_developer_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/immutable-compiler-source-developer-v2-20261117.exit"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR="$RUN/canonical-target-869a461d-request-workspace"
export TMPDIR="$RUN/canonical-tmp-869a461d-request-workspace" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --profile "$RUN/tooling-profile-immutable-compiler-source-dev" --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; cargo fmt --check -p mantle; cargo test --release --locked --offline -p mantle --bin mantle source_built_recipe_exec::tests:: -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --no-deps -- -D warnings; cargo -Zscript scripts/check-machine-schema-contracts.rs; cargo build --release --locked --offline -p mantle --bin mantle'
"$CARGO_TARGET_DIR/release/mantle" --version
printf 'immutable_compiler_source_developer_complete=1\n'
