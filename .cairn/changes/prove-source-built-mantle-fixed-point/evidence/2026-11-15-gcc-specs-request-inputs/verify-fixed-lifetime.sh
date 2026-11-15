#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/gcc-specs-request-dev.glWdGv
test ! -e "$RUN/gcc-specs-request-fixed-lifetime.log"
exec > "$RUN/gcc-specs-request-fixed-lifetime.log" 2>&1
trap 'status=$?; printf "fixed_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR="$RUN/runner-build-target-71c2bd4d-fresh-target"
export TMPDIR="$RUN/runner-tmp-71c2bd4d-fresh-target" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --offline --builders "" --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; rustfmt --edition 2024 --config skip_children=true src/source_built_native_programs_phases.rs src/source_built_native_programs_early_gnu_generator_base.rs src/source_built_mantle_programs_native.rs; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings'
