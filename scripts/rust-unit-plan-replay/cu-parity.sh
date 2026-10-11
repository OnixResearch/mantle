#!/usr/bin/env bash
# Matched-output parity: run the unit-plan app executable and the default-lane offlineCargoPackage app with identical
# (empty) arguments and environment (env -i, cwd /); check stdout, stderr and exit against the same fixture expectation
# (stdout "", stderr "", exit 0: app/src/main.rs only asserts answer() == 42) and against each other.
set -uo pipefail
R=$HOME/mantle-cairn-finish
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
offline_run=${CU_OFFLINE_RUN:-09-offline-cargo-baseline}
D=$B/parity
mkdir "$D" || { echo "refusing to overwrite prior parity proof $D" >&2; exit 1; }
unit_logical=$(jq -er '.plan_output_bindings[] | select(.name == "app" and .status == "bound") | .output_path' "$B/runs/02-fresh-baseline-app/report.json") || exit 1
cargo_out=$(jq -er '.outcomes[] | select(.label == "unit-baseline-app-offline-cargo") | .outputs[] | select(.name == "out") | .path' "$B/runs/$offline_run/report.json") || exit 1
unit_app=$R/stores/rust-efficiency-20261011/${CU_STORE_NAMESPACE:-cu}-publisher/store/${unit_logical##*/}/lib/unit_baseline_app
cargo_app=$cargo_out/bin/unit-baseline-app
test -x "$unit_app" && test -x "$cargo_app" || { echo "paired executable missing" >&2; exit 1; }
sha256sum "$unit_app" "$cargo_app"
jq -c '{schema,claim_class,target,profile,toolchain,cargo_command}' "$cargo_out/share/mantle/offline-cargo-build.json" || exit 1
fail=0
for lane in unit-plan offline-cargo; do
  if [[ $lane == unit-plan ]]; then app=$unit_app; else app=$cargo_app; fi
  (cd / && env -i "$app" >"$D/$lane.stdout" 2>"$D/$lane.stderr"); echo $? >"$D/$lane.exit"
  printf "lane=%s exit=%s stdout_bytes=%s stderr_bytes=%s\n" "$lane" "$(cat $D/$lane.exit)" "$(wc -c <$D/$lane.stdout)" "$(wc -c <$D/$lane.stderr)"
  [[ $(cat $D/$lane.exit) == 0 && ! -s $D/$lane.stdout && ! -s $D/$lane.stderr ]] || { echo "lane $lane differs from fixture expectation"; fail=1; }
done
for part in stdout stderr exit; do cmp -s "$D/unit-plan.$part" "$D/offline-cargo.$part" && echo "$part: identical" || { echo "$part: DIFFERENT"; fail=1; }; done
echo "parity_failed=$fail"; exit $fail
