#!/usr/bin/env bash
set -euo pipefail

# Active repo-root lazy-eval session: parallel-root throughput.
# Selected-root metrics remain bundle guardrails; this script emits only the
# parallel-root primary metric in its runner summary.

readonly PRIMARY_METRIC_NAME="parallel_all_roots_total_wall_ns"
readonly PRIMARY_WORKLOAD_NAME="parallel-all-roots-wide-package-set"
readonly PRIMARY_METRIC_SECTION_LABEL="=== Primary metric ==="
readonly DEFAULT_REPEAT_COUNT="10"
readonly DEFAULT_BUNDLE_OUT="target/benchmarks/parallel-root-run.json"
readonly DEFAULT_CARGO_TARGET_ROOT="target/autoresearch-parallel-root"
readonly BASELINE_BUNDLE_OUT_LABEL="BASELINE_BUNDLE_OUT"
readonly BASELINE_CARGO_TARGET_DIR_LABEL="BASELINE_CARGO_TARGET_DIR"

REPEAT_COUNT="${REPEAT_COUNT:-$DEFAULT_REPEAT_COUNT}"
BUNDLE_OUT="${BUNDLE_OUT:-$DEFAULT_BUNDLE_OUT}"
CARGO_TARGET_ROOT="${CARGO_TARGET_ROOT:-$DEFAULT_CARGO_TARGET_ROOT}"

export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/6jafhh81cf85d0vqwrnhl5yfc4wibxvq-protobuf-29.6/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export SNIX_BUILD_SANDBOX_SHELL="${SNIX_BUILD_SANDBOX_SHELL:-/bin/sh}"

mkdir -p "$(dirname "$BUNDLE_OUT")"
mkdir -p "$CARGO_TARGET_ROOT"
CARGO_TARGET_DIR="$(mktemp -d "$CARGO_TARGET_ROOT/run-XXXXXX")"

export BUNDLE_OUT
export CARGO_TARGET_DIR
export PRIMARY_METRIC_NAME
export PRIMARY_WORKLOAD_NAME

echo "${BASELINE_BUNDLE_OUT_LABEL}=$BUNDLE_OUT"
echo "${BASELINE_CARGO_TARGET_DIR_LABEL}=$CARGO_TARGET_DIR"

cargo run --example benchmark_lazy_eval -- \
  --repeat-count "$REPEAT_COUNT" \
  --bundle-out "$BUNDLE_OUT"

echo ""
echo "$PRIMARY_METRIC_SECTION_LABEL"
python3 - <<'PY'
import json
import os
import sys

bundle_path = os.environ["BUNDLE_OUT"]
primary_metric_name = os.environ["PRIMARY_METRIC_NAME"]
primary_workload_name = os.environ["PRIMARY_WORKLOAD_NAME"]

with open(bundle_path) as f:
    results = json.load(f)
for result in results:
    if result["workload_name"] != primary_workload_name:
        continue
    for metric in result["phase_metrics"]:
        if metric["name"] == primary_metric_name:
            print(f"{primary_metric_name} = {metric['value']} ns")
            print(f"METRIC {primary_metric_name}={metric['value']}")
            sys.exit(0)
print(f"metric not found: {primary_metric_name}", file=sys.stderr)
sys.exit(1)
PY
