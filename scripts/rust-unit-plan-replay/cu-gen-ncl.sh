#!/usr/bin/env bash
# Generate explicit typed goals from signed store roots, never rewrite the
# checked-in example's historical pinned paths or silently change its inputs.
set -euo pipefail
R=$HOME/mantle-cairn-finish
WT=$R/wt/rust-efficiency-20261011
B=${CU_B:-$R/tmp/rust-efficiency-20261011/cu-unit}
HERE=$WT/scripts/rust-unit-plan-replay
tag=$1 cfg=$2 ws=$3 root=$4
p() { jq -er --arg key "$1" '.[$key]' "$B/paths.json"; }
[[ $root =~ ^u\.[0-9a-f]{64}$ ]] || { echo "invalid admitted unit root: $root" >&2; exit 1; }
mkdir -p "$B/ncl"
cat > "$B/ncl/$tag.ncl" <<EOF
let cargo_unit_plan = import "$WT/lib/cargo_unit_plan.ncl" in
let mantle = import "$WT/lib/lib.ncl" in
let plan_producer = cargo_unit_plan {
  goal_name = "cargo-unit-plan-representative",
  producer = "$(p producer)",
  config = "$cfg",
  workspace = "$ws",
  planner = "$(p planner)",
  toolchain = "$(p toolchain)",
  linker = "$(p linker)",
  helper = "$(p helper)",
  vendor_sources = [],
  runtime_inputs = ["$(p compiler)"],
} in
{
  producer = plan_producer,
  app = {
    name = "cargo-unit-plan-app-check",
    builder = "$(p app_consumer)",
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    inputs = [
      "$(p app_consumer)",
      {
        name = "app",
        producer = plan_producer,
        plan_output = "plan",
        root = "$root",
        unit_output = "out",
      },
    ],
    env = { MANTLE_UNIT_APP = "{{mantle-plan-output:app}}" },
  } | mantle.Derivation,
}
EOF
printf 'let goal = import "%s" in goal.producer\n' "$B/ncl/$tag.ncl" > "$B/ncl/$tag-producer.ncl"
if [[ $tag == baseline ]]; then
  cat > "$B/ncl/offline-cargo-baseline.ncl" <<EOF
let offline = import "$HERE/offline-cargo-baseline.ncl" in
offline {
  workspace = "$ws",
  rust_toolchain = "$(p toolchain)",
  shim = "$(p shim)",
  musl_runtime = "$(p musl)",
}
EOF
fi
echo "generated producer and app goals for $tag from signed inputs, root=$root"
