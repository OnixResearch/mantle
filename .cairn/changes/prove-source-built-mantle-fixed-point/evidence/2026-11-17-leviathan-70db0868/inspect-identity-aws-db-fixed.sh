#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
SCRATCH=/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-identity-aws-db-fixed
mkdir "$SCRATCH"
exec > "$SCRATCH/execution.log" 2>&1
trap 'status=$?; printf "identity_inspection_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
test "$(cat /home/brittonr/sbfp-20261113-berdKH/preparation-70db0868-primary-sources.exit)" = 0
/run/current-system/sw/bin/bash /home/brittonr/sbfp-20261113-berdKH/review-tools-70db0868-primary-sources/identity-review/run-full-identity.sh /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources /home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources "$SCRATCH" /home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json 70db08685c0c5fa5c326bfe32db78576d7e2e182
