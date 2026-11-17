#!/usr/bin/env bash
set -euo pipefail
BASE='.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-17-leviathan-b3a39a01'
LOG="$BASE/cairn-qualified-evidence.log"
test ! -e "$LOG"
test ! -e "$BASE/cairn-qualified-evidence-validation.json"
exec > "$LOG" 2>&1
trap 'status=$?; printf "current_cairn_exit=%s\n" "$status"; printf "%s\n" "$status" > "$BASE/cairn-qualified-evidence.exit"; exit "$status"' EXIT
set -x
nix develop --profile "${TMPDIR:-/tmp}/mantle-immutable-source-cairn-profile" --no-write-lock-file -c bash -c '
set -euo pipefail
BASE="$1"
expected_cairn=$(nix eval --raw github:OnixResearch/cairn/007ba5193dd8fec9067a1c0e0f8ef487807af872#packages.x86_64-linux.default.outPath)
actual_cairn=$(readlink -f "$(command -v cairn)")
test "$actual_cairn" = "$expected_cairn/bin/cairn"
printf "pinned_cairn=%s\n" "$actual_cairn"
b3sum "$actual_cairn"
cairn validate --root . > "$BASE/cairn-qualified-evidence-validation.json"
cat "$BASE/cairn-qualified-evidence-validation.json"
jq -e ".valid == true" "$BASE/cairn-qualified-evidence-validation.json"
' bash "$BASE"
