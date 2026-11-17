#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION='b3a39a0118839fd6f7267c9b1c99258155ff40af'
test "$('/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum' --no-names '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json')" = 'd28fbde59ae0de59479155f31a9d133c991b608ed7f7969a36dcd8c878557919'
exec /run/current-system/sw/bin/bash '/home/brittonr/sbfp-20261113-berdKH/review-tools-b3a39a01-immutable-source-route-ref-fixed/graph-review/resume-controller-ref-fixed.sh' '/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source' '/home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-graph' '/home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json'
