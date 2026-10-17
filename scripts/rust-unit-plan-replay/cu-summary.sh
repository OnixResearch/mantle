#!/usr/bin/env bash
# cu-summary.sh RUN_NAME STORE_TAG [EXPECT_BUILT EXPECT_CACHED EXPECT_FAILED]
# Prints one JSON line per run: exit, counts, plan digest and every outcome with its role (producer, lib/bin unit,
# consumer), cached flag and output apparent bytes; appends it to $B/summary.jsonl. With expectations, exits 1 on mismatch.
set -euo pipefail
R=$HOME/mantle-cairn-finish; B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
store_namespace=${CU_STORE_NAMESPACE:-cu}
run=$1 tag=$2
dir=$B/runs/$run
report=$dir/report.json
plan=$(jq -r '.native_dynamic_plans[0].plan_artifact_path // empty' "$report")
physical_plan=""
if [[ -n $plan ]]; then physical_plan=$R/stores/rust-efficiency-20261011/$store_namespace-$tag/store/${plan#/nix/store/}; fi
kinds='{}'
if [[ -f $physical_plan ]]; then
  kinds=$(jq -c '[.units[] | {key: ("rust-unit-" + (.id | ltrimstr("u.") | .[0:16]) + ".drv"),
    value: (if (.derivation | tostring | test("\"--crate-type\",\"lib\"")) then "lib-unit"
            elif (.derivation | tostring | test("\"--crate-type\",\"bin\"")) then "bin-unit" else "unit" end)}] | from_entries' "$physical_plan")
fi
outcomes=$(jq -c --argjson kinds "$kinds" '[.outcomes[] | {label, cached,
    role: (if ($kinds[.label] != null) then $kinds[.label] elif (.label | startswith("rust-unit-")) then "unit"
           elif (.label == "producer" or (.label | test("plan-representative"))) then "producer" else "consumer" end),
    outputs: [.outputs[] | .path]}]' "$report")
with_bytes='[]'
while IFS= read -r o; do
  bytes=0
  while IFS= read -r p; do
    [[ -e $p ]] && bytes=$(( bytes + $(du -sb --apparent-size "$p" | cut -f1) ))
  done < <(jq -r '.outputs[]' <<<"$o")
  with_bytes=$(jq -c --argjson o "$o" --argjson b "$bytes" '. + [$o + {output_bytes: $b} | del(.outputs)]' <<<"$with_bytes")
done < <(jq -c '.[]' <<<"$outcomes")
line=$(jq -cn --arg run "$run" --arg store "rust-efficiency-20261011/$store_namespace-$tag" --argjson exit "$(cat "$dir/exit")" --argjson wall "$(cat "$dir/wall_s")" \
  --argjson counts "$(jq -c '.counts' "$report")" \
  --argjson plans "$(jq -c '[(.native_dynamic_plans // [])[] | {canonical_plan_digest, raw_artifact_digest, accepted_unit_ids, rejection_reason}]' "$report")" \
  --argjson bindings "$(jq -c '[(.plan_output_bindings // [])[] | {root, status, failure_reason}]' "$report")" \
  --argjson outcomes "$with_bytes" '{run: $run, store: $store, exit: $exit, wall_s_diagnostic: $wall, counts: $counts,
    units_executed: [$outcomes[] | select((.role | endswith("unit")) and (.cached | not))] | length,
    units_reused: [$outcomes[] | select((.role | endswith("unit")) and .cached)] | length,
    executed_bytes: ([$outcomes[] | select(.cached | not) | .output_bytes] | add // 0),
    reused_bytes: ([$outcomes[] | select(.cached) | .output_bytes] | add // 0),
    plans: $plans, bindings: $bindings, outcomes: $outcomes}')
echo "$line" | tee -a "$B/summary.jsonl"
if [[ $# -ge 5 ]]; then
  got=$(jq -r '.counts | "\(.built_total) \(.cached_total) \(.failed_total)"' <<<"$line")
  [[ $got == "$3 $4 $5" ]] || { echo "EXPECTATION FAILED for $run: built/cached/failed=$got expected=$3 $4 $5" >&2; exit 1; }
  echo "expectation ok: $run built/cached/failed=$got"
fi
