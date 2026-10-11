#!/usr/bin/env bash
# Assertions consume only this run's real Mantle reports, accepted plan artifacts and physical outputs.
set -euo pipefail
mode=$1 B=$2 STORE=$3
report() { printf '%s/runs/%s/report.json\n' "$B" "$1"; }
root_drv_name() { # accepted root ID -> exact derivation name in this run's admitted plan
  jq -er '.roots[0] as $root | .units[] | select(.id == $root) | .derivation.name + ".drv"' "$B/runs/$1/accepted-plan.json"
}
unit_output() { # accepted root -> actual built output, never infer from a fixture name
  local tag=$1 label
  label=$(root_drv_name "$tag")
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
assert_flag() { # per-run flags are outputs of actual executed build-script units
  local tag=$1 filename=$2 pattern=$3 output
  while IFS= read -r output; do
    if [[ -f $output/$filename ]] && grep -Fq -- "$pattern" "$output/$filename"; then return 0; fi
  done < <(jq -r '.outcomes[].outputs[].path' "$(report "$tag")")
  echo "missing $tag build-script directive $filename $pattern" >&2; exit 1
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
    assert_flag links links 'DEP_FOO_INCLUDE'
    assert_flag links env 'BAR_PROFILE_NOTE'
    assert_flag links args-immediate 'bar_generated'
    assert_flag links args-immediate 'foo_sys_built'
    assert_flag links rerun 'build.rs'
    assert_flag bz-a args-transitive 'bz2'
    assert_flag bz-b args-transitive 'bz2'
    # Check the exact declared pkg-config path reached the script's flags, not a host or bundled C library.
    lib_a=$(jq -er '."registry+https://github.com/rust-lang/crates.io-index#bzip2-sys@0.1.13+1.0.8".inputs.bzip2' "$B/ncl/bz-a-native.json")
    lib_b=$(jq -er '."registry+https://github.com/rust-lang/crates.io-index#bzip2-sys@0.1.13+1.0.8".inputs.bzip2' "$B/ncl/bz-b-native.json")
    assert_flag bz-a args-transitive "$lib_a/lib"
    assert_flag bz-b args-transitive "$lib_b/lib"
    jq -en --slurpfile a "$(report bz-a)" --slurpfile b "$(report bz-b)" \
      --slurpfile original "$B/runs/bz-a/accepted-plan.json" \
      --slurpfile plan "$B/runs/bz-b/accepted-plan.json" '
      ($original[0].units | map({key:.id, value:.derivation}) | from_entries) as $prior |
      ($plan[0].units | map(select(.derivation == $prior[.id]))) as $unchanged |
      ($a[0].native_dynamic_plans[0].canonical_plan_digest != $b[0].native_dynamic_plans[0].canonical_plan_digest) and
      ($unchanged | length > 0) and
      ($unchanged | all(.[];
        .derivation.name as $name | $b[0].outcomes |
        any(.[]; .label == ($name + ".drv") and .cached == true))) and
      ($plan[0].roots[0] as $root | $plan[0].units[] | select(.id == $root) |
        (.derivation != $prior[$root]) and
        (.derivation.name as $name | $b[0].outcomes |
          any(.[]; .label == ($name + ".drv") and .cached == false))) and
      ($plan[0].units | any(.[]; (.derivation.args | tostring | contains("run-build-script")) and
        (.derivation != $prior[.id]) and
        (.derivation.name as $name | $b[0].outcomes |
          any(.[]; .label == ($name + ".drv") and .cached == false))))' >/dev/null
    ;;
  sandbox)
    for tag in host network; do
      marker=$tag-denied found=0
      while IFS= read -r output; do
        if [[ -f $output/$marker ]]; then cmp "$output/$marker" <(printf '%s' "$marker"); found=1; fi
      done < <(jq -r '.outcomes[].outputs[].path' "$(report "$tag")")
      [[ $found == 1 ]] || { echo "$tag probe did not execute inside script unit" >&2; exit 1; }
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
    grep -Eq 'StorePath|store path|/etc|unit-plan-host-path' "$B/runs/bz-host-role/stderr"
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
        undeclared_role:($bundle + "/runs/bz-wrong-role/stderr"),
        host_role:($bundle + "/runs/bz-host-role/stderr")}
      | .runs |= with_entries(.value |= {counts, accepted_plans:[.native_dynamic_plans[]? | {canonical_plan_digest,
        accepted_unit_ids,rejection_reason,scheduler_action}],outcomes:[.outcomes[]? | {label,cached,outputs}],failed}))' > "$B/summary.json"
fi
