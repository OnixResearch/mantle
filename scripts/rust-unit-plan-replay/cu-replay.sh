#!/usr/bin/env bash
# Run detached with pueue from the isolated Mantle worktree after build-tools.sh.
# Every input enters each Mantle client through a signed Nix file cache.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
H=$WT/scripts/rust-unit-plan-replay
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
MANTLE=${CU_MANTLE:-$R/targets/rust-efficiency-20261011/release/mantle}
store_namespace=${CU_STORE_NAMESPACE:-cu}
[[ $store_namespace =~ ^cu(-[a-z0-9]+)?$ ]] || { echo 'invalid isolated store namespace' >&2; exit 1; }
PUBLISHER=$R/stores/rust-efficiency-20261011/$store_namespace-publisher
CLIENT=$R/stores/rust-efficiency-20261011/$store_namespace-client
ROOT_MISSING=u.0000000000000000000000000000000000000000000000000000000000000000
cd "$WT"
p() { jq -er --arg key "$1" '.[$key]' "$B/paths.json"; }
"$H/cu-t41-prep.sh"
"$H/cu-gen-ncl.sh" baseline "$(p cfg0)" "$(p w0)" "$ROOT_MISSING"
"$H/cu-build.sh" pull publisher "$B/input-cache"
"$H/cu-build.sh" build 01-fresh-baseline-producer publisher "$B/ncl/baseline-producer.ncl"
"$H/cu-summary.sh" 01-fresh-baseline-producer publisher 3 0 0
baseline_root=$("$H/cu-root.sh" 01-fresh-baseline-producer publisher)
"$H/cu-gen-ncl.sh" baseline "$(p cfg0)" "$(p w0)" "$baseline_root"
"$H/cu-build.sh" build 02-fresh-baseline-app publisher "$B/ncl/baseline.ncl"
"$H/cu-summary.sh" 02-fresh-baseline-app publisher 1 3 0
"$H/cu-build.sh" build 03-unchanged-rerun publisher "$B/ncl/baseline.ncl"
"$H/cu-summary.sh" 03-unchanged-rerun publisher
jq -e '.counts.built_total == 0 and .counts.failed_total == 0 and
  (.plan_output_bindings | any(.status == "bound"))' "$B/runs/03-unchanged-rerun/report.json" >/dev/null
"$H/cu-gen-ncl.sh" app-edit "$(p cfg1)" "$(p w1)" "$ROOT_MISSING"
"$H/cu-build.sh" build 04-app-edit-producer publisher "$B/ncl/app-edit-producer.ncl"
"$H/cu-summary.sh" 04-app-edit-producer publisher 2 1 0
edited_root=$("$H/cu-root.sh" 04-app-edit-producer publisher)
test "$edited_root" != "$baseline_root"
"$H/cu-gen-ncl.sh" app-edit "$(p cfg1)" "$(p w1)" "$edited_root"
"$H/cu-build.sh" build 05-app-edit-app publisher "$B/ncl/app-edit.ncl"
"$H/cu-summary.sh" 05-app-edit-app publisher 1 3 0
"$H/cu-gen-ncl.sh" outside-edit "$(p cfg2)" "$(p w2)" "$baseline_root"
"$H/cu-build.sh" build 06-outside-edit publisher "$B/ncl/outside-edit.ncl"
"$H/cu-summary.sh" 06-outside-edit publisher 2 2 0
baseline_plan=$(jq -er '.native_dynamic_plans[0].canonical_plan_digest' "$B/runs/01-fresh-baseline-producer/report.json")
app_edit_plan=$(jq -er '.native_dynamic_plans[0].canonical_plan_digest' "$B/runs/04-app-edit-producer/report.json")
outside_plan=$(jq -er '.native_dynamic_plans[0].canonical_plan_digest' "$B/runs/06-outside-edit/report.json")
test "$baseline_plan" = "$outside_plan" && test "$baseline_plan" != "$app_edit_plan"
"$H/cu-gen-ncl.sh" wrong-root "$(p cfg0)" "$(p w0)" "$ROOT_MISSING"
if "$H/cu-build.sh" build 07-wrong-root publisher "$B/ncl/wrong-root.ncl"; then
  echo 'wrong-root consumer unexpectedly ran' >&2; exit 1
fi
"$H/cu-summary.sh" 07-wrong-root publisher
jq -e '.counts.built_total == 0 and .counts.failed_total == 1 and
  ([.outcomes[].label] | index("app") == null)' "$B/runs/07-wrong-root/report.json" >/dev/null
jq -e '.plan_output_bindings | any(.status == "rejected" and .failure_reason == "plan-output-root-missing")' "$B/runs/07-wrong-root/report.json" >/dev/null
"$H/cu-build.sh" build 09-offline-cargo-baseline publisher "$B/ncl/offline-cargo-baseline.ncl"
"$H/cu-summary.sh" 09-offline-cargo-baseline publisher 1 0 0
CU_OFFLINE_RUN=09-offline-cargo-baseline "$H/cu-parity.sh"
"$MANTLE" --nix-compat --store "$PUBLISHER/store" --state-dir "$PUBLISHER/state" store sign --all
nix key convert-secret-to-public < "$PUBLISHER/state/signing-key" > "$B/keys/publisher-public"
results_cache=$B/results-cache
mkdir "$results_cache"
mapfile -t physical_results < <(jq -r '.outcomes[].outputs[].path' "$B/runs/01-fresh-baseline-producer/report.json" "$B/runs/02-fresh-baseline-app/report.json" | sort -u)
selectors=()
for output in "${physical_results[@]}"; do
  test -e "$output" && selectors+=("${output##*/}")
done
test "${#selectors[@]}" -eq 4 && test "${#physical_results[@]}" -eq 4
"$MANTLE" --nix-compat --store "$PUBLISHER/store" --state-dir "$PUBLISHER/state" store push --to "$results_cache" "${selectors[@]}" >"$B/results-push.stdout" 2>"$B/results-push.stderr"
grep -Eq 'pushed=6 skipped_unsigned=0 skipped_present=0' "$B/results-push.stderr"
test ! -e "$CLIENT" || { echo "clean client already exists: $CLIENT" >&2; exit 1; }
"$H/cu-build.sh" pull client "$B/input-cache"
trust="$(p public_key),$(cat "$B/keys/publisher-public")"
"$MANTLE" --nix-compat --store "$CLIENT/store" --state-dir "$CLIENT/state" store pull --from "$results_cache" --trusted-public-keys "$trust" --all >"$B/results-pull.stdout" 2>"$B/results-pull.stderr"
grep -Eq 'imported=6 skipped_present=0 skipped_untrusted=0 skipped_hash_mismatch=0 skipped_missing_nar=0' "$B/results-pull.stderr"
CU_TRUSTED_KEYS="$trust" "$H/cu-build.sh" build 08-clean-client client "$B/ncl/baseline.ncl"
"$H/cu-summary.sh" 08-clean-client client
jq -e '.counts.built_total == 1 and .counts.failed_total == 0 and
  ([.outcomes[] | select(.label == "app" and .cached == false)] | length == 1) and
  ([.outcomes[] | select(.label | startswith("rust-unit-"))] |
    length == 2 and all(.[]; .cached == true)) and
  ([.outcomes[] | select(.label == "producer" and .cached == false)] | length == 0)' \
  "$B/runs/08-clean-client/report.json" >/dev/null
jq -e -s 'any(.[]; .run == "03-unchanged-rerun" and .units_executed == 0 and .units_reused == 2) and any(.[]; .run == "04-app-edit-producer" and .units_executed == 1 and .units_reused == 1) and any(.[]; .run == "06-outside-edit" and .units_executed == 0 and .units_reused == 2) and any(.[]; .run == "08-clean-client" and .units_executed == 0 and .units_reused == 2)' "$B/summary.jsonl" >/dev/null
printf 'rust-unit-efficiency-proof: fresh, unchanged, package-edit, outside-edit, wrong-root, matched offline Cargo behavior, signed clean client PASS\n'
