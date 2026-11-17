#!/usr/bin/env bash
set -euo pipefail
test "$#" = 1
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION='b3a39a0118839fd6f7267c9b1c99258155ff40af'
test "$(cat '/home/brittonr/sbfp-20261113-berdKH/preparation-b3a39a01-immutable-source.exit')" = 0
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json')" = "$1"
SCRATCH='/home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-graph'
mkdir -p -- "$SCRATCH"
LOG='/home/brittonr/sbfp-20261113-berdKH/inspection-helper-mutations-b3a39a01-immutable-source.log'
test ! -e "$LOG"
exec > "$LOG" 2>&1
trap 'status=$?; printf "helper_mutations_exit=%s\n" "$status"; printf "%s\n" "$status" > "/home/brittonr/sbfp-20261113-berdKH/inspection-helper-mutations-b3a39a01-immutable-source.exit"; exit "$status"' EXIT
set -x
'/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' '/home/brittonr/sbfp-20261113-berdKH/review-tools-b3a39a01-immutable-source/check-helper-mutations.py' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-graph' '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json'
