#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
exec > "$RUN/canonical-final-source-tests.log" 2>&1
trap 's=$?; printf "canonical_final_source_tests_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/canonical-final-source-tests.exit"; exit "$s"' EXIT
set -x
cd "$RUN/runner-build-source"
export TMPDIR="$RUN/runner-build-tmp" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo test --release --locked --offline -p mantle --bin mantle source_built_ -- --test-threads=1; cargo test --release --locked --offline -p mantle --lib source_built_fixed_point_trust_report::receipt_tests:: -- --test-threads=1; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings'
