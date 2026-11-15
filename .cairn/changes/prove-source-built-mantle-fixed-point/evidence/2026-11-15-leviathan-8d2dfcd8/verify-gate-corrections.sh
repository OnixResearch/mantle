#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/repair-invocation-gates-8d2dfcd8
exec > "$RUN/verify-invocation-gate-corrections.log" 2>&1
trap 'status=$?; printf "verification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/verify-invocation-gate-corrections.exit"; exit "$status"' EXIT
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-8d2dfcd8-typed-invocation" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target-8fa84260-aws" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustfmt --edition 2024 --check --config skip_children=true src/source_built_rust_programs_successors_graph.rs; cargo -Zscript scripts/check-machine-schema-contracts.rs'
