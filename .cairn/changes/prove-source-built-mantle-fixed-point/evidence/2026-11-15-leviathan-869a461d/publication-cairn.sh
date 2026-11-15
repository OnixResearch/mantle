#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/gcc-specs-request-dev.glWdGv
test ! -e "$RUN/frozen-869a461d-publication-cairn.log"
exec > "$RUN/frozen-869a461d-publication-cairn.log" 2>&1
trap 'status=$?; printf "publication_cairn_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
cd "$SOURCE"
export CARGO_TARGET_DIR="$RUN/runner-build-target-71c2bd4d-fresh-target"
export TMPDIR="$RUN/runner-tmp-71c2bd4d-fresh-target" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --offline --builders "" --no-write-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; cairn validate --root .; cairn gate proposal prove-source-built-mantle-fixed-point --root .; cairn gate design prove-source-built-mantle-fixed-point --root .; cairn gate tasks prove-source-built-mantle-fixed-point --root .'
