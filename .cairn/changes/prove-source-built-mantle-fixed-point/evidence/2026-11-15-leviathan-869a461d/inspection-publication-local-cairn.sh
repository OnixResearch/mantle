#!/usr/bin/env bash
set -euo pipefail
cd /home/brittonr/git/OnixResearch/mantle
test ! -e .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-869a461d/inspection-publication-local-cairn.log
exec > .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-869a461d/inspection-publication-local-cairn.log 2>&1
trap 'status=$?; printf "inspection_publication_local_cairn_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
nix develop --builders '' --no-update-lock-file -c bash -c 'set -euo pipefail; unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER; command -v cairn; readlink -f "$(command -v cairn)"; cairn validate --root .; cairn gate proposal prove-source-built-mantle-fixed-point --root .; cairn gate design prove-source-built-mantle-fixed-point --root .; cairn gate tasks prove-source-built-mantle-fixed-point --root .'
