#!/usr/bin/env bash
# Print the one accepted binary unit identity from an actual native Mantle report.
set -euo pipefail
R=$HOME/mantle-cairn-finish
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
report=$B/runs/$1/report.json
store=$R/stores/rust-efficiency-20261011/${CU_STORE_NAMESPACE:-cu}-$2/store
logical=$(jq -er '[.native_dynamic_plans[] | select(.mode == "native" and .output_name == "plan" and .rejection_reason == null and (.accepted_unit_ids | length) == 2) | .plan_artifact_path] | if length == 1 then .[0] else error("expected one accepted two-unit native plan") end' "$report")
plan=$store/${logical##*/}
test -f "$plan" || { echo "missing accepted plan $plan" >&2; exit 1; }
root=$(jq -er '[.units[] | select(.derivation.args | index("unit_baseline_app") != null) | .id] | if length == 1 then .[0] else error("expected one baseline app unit") end' "$plan")
jq -e --arg root "$root" '.native_dynamic_plans[0].accepted_unit_ids | index($root) != null' "$report" >/dev/null
printf '%s\n' "$root"
