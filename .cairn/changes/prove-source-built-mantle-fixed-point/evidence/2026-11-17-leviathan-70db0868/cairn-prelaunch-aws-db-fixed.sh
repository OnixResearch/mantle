#!/usr/bin/env bash
set -euo pipefail
cd /home/brittonr/git/OnixResearch/mantle
BASE=.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-17-leviathan-70db0868
LOG="$BASE/cairn-prelaunch-aws-db-fixed.log"
test ! -e "$LOG"
exec > "$LOG" 2>&1
trap 'status=$?; printf "prelaunch_cairn_exit=%s\n" "$status"; printf "%s\n" "$status" > "$BASE/cairn-prelaunch-aws-db-fixed.exit"; exit "$status"' EXIT
set -x
nix develop --profile "${TMPDIR:-/tmp}/mantle-primary-source-cairn-profile" --no-write-lock-file -c bash -c '
set -euo pipefail
BASE="$1"
expected_cairn=$(nix eval --raw github:OnixResearch/cairn/007ba5193dd8fec9067a1c0e0f8ef487807af872#packages.x86_64-linux.default.outPath)
actual_cairn=$(readlink -f "$(command -v cairn)")
test "$actual_cairn" = "$expected_cairn/bin/cairn"
printf "pinned_cairn=%s\n" "$actual_cairn"
b3sum "$actual_cairn"
check_report() {
  local label="$1"
  shift
  local report="$BASE/cairn-prelaunch-aws-db-fixed-$label.json"
  test ! -e "$report"
  "$@" > "$report"
  cat "$report"
  jq -e ".valid == true" "$report"
}
check_report validation cairn validate --root .
check_report proposal cairn gate proposal prove-source-built-mantle-fixed-point --root .
check_report design cairn gate design prove-source-built-mantle-fixed-point --root .
check_report tasks cairn gate tasks prove-source-built-mantle-fixed-point --root .
' bash "$BASE"
