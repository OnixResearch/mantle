#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/helper-source-authority-dev
exec > "$RUN/helper-and-feature-authority-dev.log" 2>&1
trap 'status=$?; printf "helper_authority_dev_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/helper-and-feature-authority-dev.exit"; exit "$status"' EXIT
cd "$SOURCE"
export CARGO_TARGET_DIR=/home/brittonr/sbfp-20261113-berdKH/runner-build-target-71c2bd4d-fresh-target
export TMPDIR="$RUN/runner-tmp-71c2bd4d-fresh-target" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; rustfmt --edition 2024 --config skip_children=true src/source_built_fixed_point.rs src/source_built_recipe.rs src/source_built_rust_programs_minicargo_resolve.rs src/source_built_rust_programs_minicargo_resolve_tests.rs; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings'
printf '%s\n' 'Development regression only: no regular runner installed; existing c9803076 frozen runner and b49c087a frozen source remain unchanged. A final frozen proof runner still requires isolated absent-target qualification.'
