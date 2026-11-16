#!/usr/bin/env bash
set -euo pipefail
test ! -e /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/contract-inspection.log
exec > /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/contract-inspection.log 2>&1
trap 'status=$?; printf "contract_inspection_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/graph-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/review-subject.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/graph-audit.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/source-binding-kind-inventory.json
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/controller-template-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/review-subject.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-contract-869a461d-request-workspace.15mnSN/controller-template-audit.json
printf 'contract_inspection_complete=1\n'
