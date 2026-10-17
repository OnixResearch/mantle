#!/usr/bin/env bash
# cu-build.sh pull STORE_TAG CACHE_DIR            : fresh-or-existing isolated store imports a signed cache (--all)
# cu-build.sh build RUN_NAME STORE_TAG NCL_FILE    : mantle --json --nix-compat build --no-substitute; keeps report/stderr/exit/wall
# Run from the isolated published-source worktree; each run gets a unique name and state.
set -uo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
MANTLE=${CU_MANTLE:-$R/targets/rust-efficiency-20261011/release/mantle}
cd "$WT"
export SNIX_BUILD_BWRAP=/nix/store/vvxfl9d1b9a944ca7miswhnxg06cs6jl-mantle-wasm-component-toolchain-v1/bin/bwrap
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/h75l4bwd606k94l22ajqdb98vfar5x66-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
PUBLIC_KEY=$(cat "$B/keys/public")
store_namespace=${CU_STORE_NAMESPACE:-cu}
store_args() { printf '%s\n' --nix-compat --store "$R/stores/rust-efficiency-20261011/$store_namespace-$1/store" --state-dir "$R/stores/rust-efficiency-20261011/$store_namespace-$1/state"; }
mode=$1; shift
case $mode in
  pull)
    tag=$1 cache=$2
    mkdir -p "$R/stores/rust-efficiency-20261011/$store_namespace-$tag/store" "$R/stores/rust-efficiency-20261011/$store_namespace-$tag/state"
    mapfile -t sa < <(store_args "$tag")
    echo "$ $MANTLE ${sa[*]} store pull --from $cache --trusted-public-keys $PUBLIC_KEY --all"
    "$MANTLE" "${sa[@]}" store pull --from "$cache" --trusted-public-keys "$PUBLIC_KEY" --all
    ;;
  build)
    name=$1 tag=$2 ncl=$3
    out=$B/runs/$name
    mkdir -p "$B/runs"
    mkdir "$out" || { echo "refusing to overwrite prior proof run $out" >&2; exit 1; }
    cp "$ncl" "$out/build.ncl"
    mapfile -t sa < <(store_args "$tag")
    if [[ -n ${CU_SUBSTITUTERS:-} ]]; then sub=(--substituters "$CU_SUBSTITUTERS"); else sub=(--no-substitute); fi; if [[ -n ${CU_TRUSTED_KEYS:-} ]]; then sub+=(--trusted-public-keys "$CU_TRUSTED_KEYS"); fi
    echo "$ $MANTLE --json ${sa[*]} build ${sub[*]} -I $WT/lib $ncl"
    start=$(date +%s)
    "$MANTLE" --json "${sa[@]}" build "${sub[@]}" -I "$WT/lib" "$ncl" >"$out/report.json" 2>"$out/stderr"
    rc=$?
    echo "$rc" >"$out/exit"; echo $(( $(date +%s) - start )) >"$out/wall_s"
    echo "run=$name exit=$rc wall_s=$(cat "$out/wall_s") stderr_bytes=$(wc -c <"$out/stderr")"
    jq -c '.counts' "$out/report.json" 2>/dev/null || echo "build report unavailable; inspect $out/stderr"
    jq -c '.native_dynamic_plans[]? | {canonical_plan_digest, accepted_unit_ids, scheduler_action, rejection_reason}' "$out/report.json" 2>/dev/null
    jq -c '.plan_output_bindings[]?' "$out/report.json" 2>/dev/null
    jq -c '.outcomes[]? | {name: (.name // .drv_name // .drv_path), cached}' "$out/report.json" 2>/dev/null
    exit "$rc"
    ;;
  *) echo "unknown mode $mode" >&2; exit 2 ;;
esac
