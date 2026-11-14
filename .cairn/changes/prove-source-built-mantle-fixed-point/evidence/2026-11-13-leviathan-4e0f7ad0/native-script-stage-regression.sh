#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
exec > "$RUN/native-script-stage-regression.log" 2>&1
trap 's=$?; printf "native_script_stage_regression_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/native-script-stage-regression.exit"; exit "$s"' EXIT
set -x
cd "$RUN/canonical-source-4e0f7ad0"
/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum src/source_built_rust_programs_llvm.rs src/source_built_rust_programs_native_smoke_tests.rs
export TMPDIR="$RUN/canonical-tmp-4e0f7ad0" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo test --locked --offline -p mantle --bin mantle native_script_graphs_admit_owning_stage_products_and_reject_cross_stage_inputs -- --test-threads=1'
