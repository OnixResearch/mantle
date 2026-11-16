#!/usr/bin/env bash
set -euo pipefail
test ! -e /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/resolved-inspection.log
exec > /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/resolved-inspection.log 2>&1
trap 'status=$?; printf "resolved_inspection_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/graph-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/review-subject.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/graph-audit.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/source-binding-kind-inventory.json
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/controller-template-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/review-subject.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-resolved-869a461d-request-workspace.tON71S/controller-template-audit.json
printf 'resolved_inspection_complete=1\n'
