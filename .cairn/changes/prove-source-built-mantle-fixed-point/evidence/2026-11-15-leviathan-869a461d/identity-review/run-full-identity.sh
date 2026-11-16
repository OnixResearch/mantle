#!/usr/bin/env bash
set -euo pipefail
test ! -e /home/brittonr/sbfp-20261113-berdKH/physical-review-869a461d-request-workspace.7oAA4w/full-identity.log
exec > /home/brittonr/sbfp-20261113-berdKH/physical-review-869a461d-request-workspace.7oAA4w/full-identity.log 2>&1
trap 'status=$?; printf "inert_physical_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 -u /home/brittonr/sbfp-20261113-berdKH/physical-review-869a461d-request-workspace.7oAA4w/identity-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/physical-review-869a461d-request-workspace.7oAA4w /home/brittonr/sbfp-20261113-berdKH/physical-review-869a461d-request-workspace.7oAA4w/review-subject.json
printf 'inert_physical_complete=1\n'
