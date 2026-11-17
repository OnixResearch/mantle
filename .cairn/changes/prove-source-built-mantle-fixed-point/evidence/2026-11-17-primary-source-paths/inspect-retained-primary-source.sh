#!/usr/bin/env bash
set -euo pipefail
RUN='/home/brittonr/sbfp-20261113-berdKH'
SCRATCH='/home/brittonr/sbfp-20261113-berdKH/primary-source-admission-regression-b3a39a01-20261117'
test ! -e "$SCRATCH/execution.log"
exec > "$SCRATCH/execution.log" 2>&1
trap 'status=$?; printf "retained_primary_source_regression_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1 EXPECTED_FROZEN_PRODUCT_REVISION='b3a39a0118839fd6f7267c9b1c99258155ff40af' EXPECTED_REVIEW_SUBJECT_BLAKE3='d28fbde59ae0de59479155f31a9d133c991b608ed7f7969a36dcd8c878557919'
'/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' '/home/brittonr/sbfp-20261113-berdKH/review-tools-primary-source-paths-20261117/graph-review/mantle-selector-audit.py' '/home/brittonr/sbfp-20261113-berdKH/review-tools-primary-source-paths-20261117/check-primary-source-admission.py'
'/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' '/home/brittonr/sbfp-20261113-berdKH/review-tools-primary-source-paths-20261117/check-primary-source-admission.py' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source' "$SCRATCH" '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json' '/home/brittonr/sbfp-20261113-berdKH/review-tools-primary-source-paths-20261117/graph-review/mantle-selector-audit.py'
test -s "$SCRATCH/primary-source-admission-regression.json"
printf 'retained_primary_source_regression_complete=1\n'
