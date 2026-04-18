#!/usr/bin/env bash
set -euo pipefail

# Autoresearch entry point for parallel root evaluation.
# Runs the lazy benchmark bundle with median-based repeated sampling and
# prints the primary metric (parallel_all_roots_total_wall_ns).

REPEAT_COUNT="${REPEAT_COUNT:-10}"
BUNDLE_OUT="${BUNDLE_OUT:-openspec/changes/parallel-root-evaluation/evidence/parallel-root-benchmark.json}"
: "${CARGO_TARGET_DIR:=target/autoresearch-parallel-root}"

export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/6jafhh81cf85d0vqwrnhl5yfc4wibxvq-protobuf-29.6/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export SNIX_BUILD_SANDBOX_SHELL="${SNIX_BUILD_SANDBOX_SHELL:-/bin/sh}"

mkdir -p "$(dirname "$BUNDLE_OUT")"
export BUNDLE_OUT
export CARGO_TARGET_DIR

cargo run --example benchmark_lazy_eval -- \
  --repeat-count "$REPEAT_COUNT" \
  --bundle-out "$BUNDLE_OUT"

echo ""
echo "=== Primary metric ==="
python3 - <<'PY'
import json
import os
import sys

bundle_path = os.environ["BUNDLE_OUT"]
with open(bundle_path) as f:
    results = json.load(f)
for result in results:
    if result["workload_name"] != "parallel-all-roots-wide-package-set":
        continue
    for metric in result["phase_metrics"]:
        if metric["name"] == "parallel_all_roots_total_wall_ns":
            print(f"parallel_all_roots_total_wall_ns = {metric['value']} ns")
            print(f"METRIC parallel_all_roots_total_wall_ns={metric['value']}")
            sys.exit(0)
print("metric not found", file=sys.stderr)
sys.exit(1)
PY
