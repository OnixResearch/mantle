#!/usr/bin/env bash
set -euo pipefail
test "$#" = 1
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION='70db08685c0c5fa5c326bfe32db78576d7e2e182'
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json')" = "$1"
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/check-seed-source-regressions-70db0868-primary-sources.py')" = '79223ac30a0e0aa96c23c2957c80adce2b40ab3d0730575b27ceb07edaf90c4e'
test ! -e '/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-70db0868-primary-sources-aws-db-fixed.log'
exec > '/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-70db0868-primary-sources-aws-db-fixed.log' 2>&1
trap 'status=$?; printf "seed_source_regressions_exit=%s\n" "$status"; printf "%s\n" "$status" > "/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-70db0868-primary-sources-aws-db-fixed.exit"; exit "$status"' EXIT
set -x
'/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' '/home/brittonr/sbfp-20261113-berdKH/check-seed-source-regressions-70db0868-primary-sources.py' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources' '/home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources' '/home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-graph-aws-db-fixed' '/home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json' '/home/brittonr/sbfp-20261113-berdKH/review-tools-70db0868-primary-sources'
