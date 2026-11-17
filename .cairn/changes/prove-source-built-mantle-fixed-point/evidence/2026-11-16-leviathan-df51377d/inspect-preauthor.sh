#!/usr/bin/env bash
set -euo pipefail
SCRATCH='/home/brittonr/sbfp-20261113-berdKH/inspection-df51377d-unit-plans-preauthor'
test ! -e "$SCRATCH"
mkdir "$SCRATCH"
exec > "$SCRATCH/execution.log" 2>&1
trap 'status=$?; printf "inspection_exit=%s\n" "$status"; exit "$status"' EXIT
/run/current-system/sw/bin/bash '/home/brittonr/sbfp-20261113-berdKH/review-tools-df51377d-unit-plans/identity-review/run-preauthor-roles.sh' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-df51377d-unit-plans' '/home/brittonr/sbfp-20261113-berdKH/runner-source-df51377d-unit-plans' '/home/brittonr/sbfp-20261113-berdKH/inspection-df51377d-unit-plans-preauthor' '/home/brittonr/sbfp-20261113-berdKH/review-subject-df51377d-unit-plans.json'
