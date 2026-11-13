#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
exec > "$RUN/canonical-checks.log" 2>&1
trap 's=$?; printf "canonical_checks_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/canonical-checks.exit"; exit "$s"' EXIT
set -x
cd "$RUN/source-checkout"
mkdir -p "$RUN/canonical-tmp"
export TMPDIR="$RUN/canonical-tmp" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/canonical-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c /run/current-system/sw/bin/bash "$RUN/canonical-checks-inside.sh"
