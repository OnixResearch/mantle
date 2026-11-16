#!/usr/bin/env bash
set -euo pipefail
SOURCE=/home/brittonr/sbfp-20261113-berdKH/inspection-publication-869a461d-request-workspace.qycVlZ
test ! -e /home/brittonr/sbfp-20261113-berdKH/inspection-publication-869a461d-request-workspace.qycVlZ/inspection-publication-cairn.log
exec > /home/brittonr/sbfp-20261113-berdKH/inspection-publication-869a461d-request-workspace.qycVlZ/inspection-publication-cairn.log 2>&1
trap 'status=$?; printf "inspection_publication_cairn_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR=/home/brittonr/sbfp-20261113-berdKH/canonical-target-869a461d-request-workspace
export TMPDIR=/home/brittonr/sbfp-20261113-berdKH/runner-tmp-71c2bd4d-fresh-target XDG_CACHE_HOME=/home/brittonr/sbfp-20261113-berdKH/tooling-cache CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
/run/current-system/sw/bin/nix develop --offline --builders "" --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; command -v cairn; readlink -f "$(command -v cairn)"; cairn validate --root .; cairn gate proposal prove-source-built-mantle-fixed-point --root .; cairn gate design prove-source-built-mantle-fixed-point --root .; cairn gate tasks prove-source-built-mantle-fixed-point --root .'
