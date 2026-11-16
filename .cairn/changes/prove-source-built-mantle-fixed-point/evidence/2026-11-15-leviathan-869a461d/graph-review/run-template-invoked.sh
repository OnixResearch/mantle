#!/usr/bin/env bash
set -euo pipefail
test ! -e /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs/template-invoked.log
exec > /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs/template-invoked.log 2>&1
trap 'status=$?; printf "template_inspector_exit=%s\n" "$status"; exit "$status"' EXIT
set -x
export PYTHONDONTWRITEBYTECODE=1
/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs/controller-template-audit.py /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/runner-source-869a461d-request-workspace /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs/review-subject.json
test -s /home/brittonr/sbfp-20261113-berdKH/inert-review-869a461d-request-workspace.j2KYjs/controller-template-audit.json
printf 'template_inspector_complete=1\n'
