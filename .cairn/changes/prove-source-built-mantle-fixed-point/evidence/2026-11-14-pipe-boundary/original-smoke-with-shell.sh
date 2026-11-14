#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH/pipe-boundary-validation.YinuAM
exec > "$ROOT/original-smoke-with-shell.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "original_smoke_exit=%s\n" "$status"; printf "%s\n" "$status" > "$ROOT/original-smoke-with-shell.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
cd "$ROOT/source"
export TMPDIR="$ROOT/tmp" XDG_CACHE_HOME="/home/brittonr/sbfp-20261113-berdKH/tooling-cache"
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash "$ROOT/original-smoke-with-shell-inside.sh"
/run/current-system/sw/bin/jq -e '.status == "passed" and (.cases | length) == 38 and ([.cases[].equal] | all)' "$ROOT/smoke-work-with-shell/results/equivalence.json"
date -Is
finished=1
printf 'original_smoke_complete=1\n'
