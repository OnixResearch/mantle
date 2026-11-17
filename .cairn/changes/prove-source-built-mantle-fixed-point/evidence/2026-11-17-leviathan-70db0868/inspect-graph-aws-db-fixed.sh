#!/usr/bin/env bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
export EXPECTED_FROZEN_PRODUCT_REVISION=70db08685c0c5fa5c326bfe32db78576d7e2e182
test "$(cat /home/brittonr/sbfp-20261113-berdKH/preparation-70db0868-primary-sources.exit)" = 0
exec /run/current-system/sw/bin/bash /home/brittonr/sbfp-20261113-berdKH/review-tools-70db0868-primary-sources/graph-review/run-semantic-root-inspection.sh /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources /home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources /home/brittonr/sbfp-20261113-berdKH/inspection-70db0868-primary-sources-graph-aws-db-fixed /home/brittonr/sbfp-20261113-berdKH/review-subject-70db0868-primary-sources-aws-db-fixed.json
