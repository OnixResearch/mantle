#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION='b3a39a0118839fd6f7267c9b1c99258155ff40af'
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json')" = 'd28fbde59ae0de59479155f31a9d133c991b608ed7f7969a36dcd8c878557919'
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/check-seed-source-regressions-b3a39a01-immutable-source.py')" = '79223ac30a0e0aa96c23c2957c80adce2b40ab3d0730575b27ceb07edaf90c4e'
test ! -e '/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-b3a39a01-immutable-source.log'
exec > '/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-b3a39a01-immutable-source.log' 2>&1
trap 'status=$?; printf "seed_source_regressions_exit=%s\n" "$status"; printf "%s\n" "$status" > "/home/brittonr/sbfp-20261113-berdKH/inspection-seed-source-regressions-v2-b3a39a01-immutable-source.exit"; exit "$status"' EXIT
set -x
'/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' '/home/brittonr/sbfp-20261113-berdKH/check-seed-source-regressions-b3a39a01-immutable-source.py' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-graph' '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json' '/home/brittonr/sbfp-20261113-berdKH/review-tools-b3a39a01-immutable-source-complete-coverage'
