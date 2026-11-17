#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
SCRATCH=/home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-identity
mkdir "$SCRATCH"
exec > "$SCRATCH/execution.log" 2>&1
trap 'status=$?; printf "identity_inspection_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
test "$(cat /home/brittonr/sbfp-20261113-berdKH/preparation-b3a39a01-immutable-source.exit)" = 0
/run/current-system/sw/bin/bash /home/brittonr/sbfp-20261113-berdKH/review-tools-b3a39a01-immutable-source/identity-review/run-full-identity.sh /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source /home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source "$SCRATCH" /home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json b3a39a0118839fd6f7267c9b1c99258155ff40af
