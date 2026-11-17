#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION=b3a39a0118839fd6f7267c9b1c99258155ff40af
test "$(cat /home/brittonr/sbfp-20261113-berdKH/preparation-b3a39a01-immutable-source.exit)" = 0
exec /run/current-system/sw/bin/bash /home/brittonr/sbfp-20261113-berdKH/review-tools-b3a39a01-immutable-source/graph-review/run-semantic-root-inspection.sh /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source /home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source /home/brittonr/sbfp-20261113-berdKH/inspection-b3a39a01-immutable-source-graph /home/brittonr/sbfp-20261113-berdKH/review-subject-b3a39a01-immutable-source.json
