#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/reproduce-profile-admission-e9a58881-before
exec > "$RUN/reproduce-profile-admission-e9a58881-before.log" 2>&1
trap 'status=$?; printf "regression_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/reproduce-profile-admission-e9a58881-before.exit"; exit "$status"' EXIT
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-e9a58881-typed-index" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c cargo test --release --locked --offline -p mantle --bin mantle original_rpath_profile_survives_strict_deferred_invocation_admission -- --test-threads=1
