#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/git/OnixResearch/mantle
EVIDENCE="$ROOT/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-16-leviathan-df51377d"
test "$PWD" = "$ROOT"
test ! -e "$EVIDENCE/prelaunch-cairn.log"
exec > "$EVIDENCE/prelaunch-cairn.log" 2>&1
trap 'status=$?; printf "prelaunch_cairn_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
date -Is
cairn_report() {
    local label="$1"
    shift
    local report="$EVIDENCE/prelaunch-cairn-$label.json"
    test ! -e "$report"
    "$@" > "$report"
    cat "$report"
    jq -e '.valid == true' "$report"
}
cairn_report validation cairn validate --root .
cairn_report proposal cairn gate proposal prove-source-built-mantle-fixed-point --root .
cairn_report design cairn gate design prove-source-built-mantle-fixed-point --root .
cairn_report tasks cairn gate tasks prove-source-built-mantle-fixed-point --root .
date -Is
