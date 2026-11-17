#!/usr/bin/env bash
set -euo pipefail
test "$#" = 1
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION='70db08685c0c5fa5c326bfe32db78576d7e2e182'
test "$(cat '/home/brittonr/sbfp-20261113-berdKH/preparation-70db0868-primary-sources.exit')" = 0
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json')" = "$1"
SCRATCH='/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-selector-mutations-aws-db-fixed'
mkdir -p -- "$SCRATCH"
LOG='/home/brittonr/sbfp-20261113-berdKH/inspection-selector-mutations-70db0868-primary-sources-aws-db-fixed.log'
test ! -e "$LOG"
exec > "$LOG" 2>&1
trap 'status=$?; printf "selector_mutations_exit=%s\n" "$status"; printf "%s\n" "$status" > "/home/brittonr/sbfp-20261113-berdKH/inspection-selector-mutations-70db0868-primary-sources-aws-db-fixed.exit"; exit "$status"' EXIT
set -x
'/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' '/home/brittonr/sbfp-20261113-berdKH/review-tools-70db0868-primary-sources/check-selector-mutations.py' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources' '/home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources' '/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-selector-mutations-aws-db-fixed' '/home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json'
