#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/operator-docs-71c2bd4d-fresh-target
exec > "$RUN/prepared-cohort-docs-gates-71c2bd4d-fresh-target.log" 2>&1
trap 'status=$?; printf "operator_docs_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/prepared-cohort-docs-gates-71c2bd4d-fresh-target.exit"; exit "$status"' EXIT
cd "$SOURCE"
export TMPDIR="$RUN/canonical-tmp-71c2bd4d-fresh-target" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; cairn validate --root .; cairn gate proposal prove-source-built-mantle-fixed-point --root .; cairn gate design prove-source-built-mantle-fixed-point --root .; cairn gate tasks prove-source-built-mantle-fixed-point --root .'
