#!/usr/bin/env bash
set -euo pipefail
if (( $# != 5 )); then
  printf 'usage: %s COHORT FROZEN_SOURCE REVIEW_SCRATCH SUBJECT_JSON SEALED_EXPECTED_REVISION\n' "$0" >&2
  exit 2
fi
script_dir="$(dirname -- "$(realpath -- "${BASH_SOURCE[0]}")")"
python="$(/run/current-system/sw/bin/jq -er '.inert_tools.python | select(type == "string" and startswith("/"))' -- "$4")"
export PYTHONDONTWRITEBYTECODE=1
exec "$python" -u "$script_dir/preauthor-role-physical-audit.py" "$@"
