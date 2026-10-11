#!/usr/bin/env bash
# Assertions consume only this run's real Mantle reports, accepted plan artifacts and physical outputs.
set -euo pipefail
mode=$1 B=$2
report() { printf '%s/runs/%s/report.json\n' "$B" "$1"; }
root_drv_name() { # accepted root ID -> exact derivation name in this run's admitted plan
  jq -er '.roots[0] as $root | .units[] | select(.id == $root) | .derivation.name + ".drv"' "$B/runs/$1/accepted-plan.json"
}
app_root_drv_name() { # discover the unique executable among actual accepted root outputs
  local tag=$1 label out artifact selected= matches=0
  while IFS= read -r label; do
    out=$(jq -er --arg label "$label" '.outcomes[] | select(.label == $label) | .outputs[] | select(.name == "out") | .path' "$(report "$tag")") || continue
    test -f "$out/share/mantle/unit-dependencies-v1.json" || continue
    artifact=$(jq -r '.artifacts[] | select(startswith("lib/") and ((endswith(".d") or endswith(".rmeta") or endswith(".rlib") or endswith(".so")) | not))' "$out/share/mantle/unit-dependencies-v1.json")
    if [[ $artifact == lib/* && $artifact != *..* ]] && test -x "$out/$artifact"; then
      selected=$label
      matches=$((matches + 1))
    fi
  done < <(jq -r '.roots as $roots | .units[] | select(.id as $id | $roots | index($id) != null) | .derivation.name + ".drv"' "$B/runs/$tag/accepted-plan.json")
  [[ $matches == 1 ]] || { echo "expected one executable accepted root for $tag; found $matches" >&2; return 1; }
  printf '%s\n' "$selected"
}
unit_output() { # accepted root -> actual built output, never infer from a fixture name
  local tag=$1 label
  label=$(app_root_drv_name "$tag")
  jq -er --arg label "$label" '.outcomes[] | select(.label == $label) | .outputs[] | select(.name == "out") | .path' "$(report "$tag")"
}
app_stdout() {
  local tag=$1 out artifact exe
  out=$(unit_output "$tag")
  test -f "$out/share/mantle/unit-dependencies-v1.json"
  artifact=$(jq -er '.artifacts | map(select((endswith(".d") or endswith(".rmeta")) | not)) | if length == 1 then .[0] else error("expected one executable") end' "$out/share/mantle/unit-dependencies-v1.json")
  [[ $artifact == lib/* && $artifact != *..* ]]
  exe=$out/$artifact
  test -x "$exe"
  "$exe" > "$B/runs/$tag/app.stdout" # host parity only; sandbox proof is the accepted Mantle plan and build outcome
}
case $mode in
  positive)
    for tag in graphs links bz-a bz-b; do app_stdout "$tag"; done
    printf 'tokens=a + b id=7 strlen=3\n' > "$B/graphs-expected.stdout"
    printf 'foo:/declared/foo/include:generated\n' > "$B/links-expected.stdout"
    printf 'bz2-roundtrip=mantle unit plan bzip2 fixture packed_bytes=66\n' > "$B/bz-expected.stdout"
    for tag in graphs links bz-a; do
      if [[ $tag == bz-a ]]; then baseline=bz; else baseline=$tag; fi
      cmp "$B/runs/$tag/app.stdout" "$B/offline/$baseline.stdout"
      cmp "$B/runs/$tag/app.stdout" "$B/$baseline-expected.stdout"
    done
    cmp "$B/runs/bz-b/app.stdout" "$B/bz-expected.stdout"
    jq -en --slurpfile a "$(report bz-a)" --slurpfile b "$(report bz-b)" \
      --slurpfile original "$B/runs/bz-a/accepted-plan.json" \
      --slurpfile plan "$B/runs/bz-b/accepted-plan.json" '
      ($original[0].units | map({key:.id, value:.derivation}) | from_entries) as $prior |
      ($plan[0].units | map(select(.derivation == $prior[.id]))) as $unchanged |
      ($a[0].native_dynamic_plans[0].canonical_plan_digest != $b[0].native_dynamic_plans[0].canonical_plan_digest) and
      ($unchanged | length > 0) and
      ($unchanged | all(.[];
        .derivation.name as $name | $b[0].outcomes |
        all(.[]; .label != ($name + ".drv") or .cached == true))) and
      ($plan[0].roots[0] as $root | $plan[0].units[] | select(.id == $root) |
        (.derivation != $prior[$root]) and
        (.derivation.name as $name | $b[0].outcomes |
          any(.[]; .label == ($name + ".drv") and .cached == false))) and
      ($plan[0].units | any(.[]; (.derivation.args | tostring | contains("run-build-script")) and
        (.derivation != $prior[.id])))' >/dev/null
    ;;
  sandbox)
    for tag in host network; do
      app_stdout "$tag"
      cmp "$B/runs/$tag/app.stdout" "$B/links-expected.stdout"
      jq -e --arg root "$(app_root_drv_name "$tag")" --slurpfile baseline "$(report links)" '
        .counts.failed_total == 0 and
        .native_dynamic_plans[0].canonical_plan_digest != $baseline[0].native_dynamic_plans[0].canonical_plan_digest and
        ([.outcomes[] | select(.label == $root and .cached == false)] | length == 1)' \
        "$(report "$tag")" >/dev/null
    done
    ;;
  negatives)
    nonzero_root=$(root_drv_name nonzero)
    jq -e --arg root "$nonzero_root" '.counts.failed_total > 0 and
      ([.outcomes[].label] | index($root) == null)' "$(report nonzero)" >/dev/null
    undeclared_root=$(root_drv_name bz-undeclared)
    jq -e --arg root "$undeclared_root" '.counts.failed_total > 0 and
      .native_dynamic_plans[0].scheduler_action == "registered-roots" and
      ([.outcomes[].label] | index($root) == null)' "$(report bz-undeclared)" >/dev/null
    jq -e '[.units[].derivation | tostring | contains("run-build-script")] | any' "$B/runs/bz-undeclared/accepted-plan.json" >/dev/null
    grep -R -Eiq 'pkg.?config|failed to find tool|could not find.*(cc|gcc)|no such file or directory' "$B/runs/bz-undeclared/logs"
    jq -s -e 'all(.[]; .counts.failed_total > 0 and (.native_dynamic_plans | length == 0))' \
      "$(report bz-wrong-role)" "$(report bz-host-role)" >/dev/null
    jq -e '(.native_dynamic_plans | length) == 0 and (.failed | any(.[]; .message | test("StorePath|store path|/etc|unit-plan-host-path")))' \
      "$(report bz-host-role)" >/dev/null
    ;;
  *) echo "invalid check mode: $mode" >&2; exit 2 ;;
esac
if [[ $mode == negatives ]]; then
  jq -n --slurpfile graphs "$(report graphs)" --slurpfile links "$(report links)" \
    --slurpfile a "$(report bz-a)" --slurpfile b "$(report bz-b)" \
    --slurpfile nonzero "$(report nonzero)" --slurpfile host "$(report host)" \
    --slurpfile network "$(report network)" --slurpfile absent "$(report bz-undeclared)" \
    --slurpfile wrong "$(report bz-wrong-role)" --slurpfile path "$(report bz-host-role)" --arg bundle "$B" \
    '{schema:"mantle-build-script-replay-v1", runs: {graphs:$graphs[0],links:$links[0],bz_a:$a[0],bz_b:$b[0],
      nonzero:$nonzero[0],host:$host[0],network:$network[0],undeclared:$absent[0],wrong_role:$wrong[0],host_role:$path[0]},
      diagnostic_files: {
        build_script_nonzero:($bundle + "/runs/nonzero/logs"),
        missing_native:($bundle + "/runs/bz-undeclared/logs"),
        undeclared_role:($bundle + "/runs/bz-wrong-role/report.json"),
        host_role:($bundle + "/runs/bz-host-role/report.json")}}
      | .runs |= with_entries(.value |= {counts, accepted_plans:[.native_dynamic_plans[]? | {canonical_plan_digest,
        accepted_unit_ids,rejection_reason,scheduler_action}],outcomes:[.outcomes[]? | {label,cached,outputs}],failed})' > "$B/summary.json"
fi
