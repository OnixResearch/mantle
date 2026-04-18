#!/usr/bin/env bash
set -euo pipefail

# Autoresearch entry point for lazy root evaluation.
# Runs the lazy benchmark with median-based repeated sampling and
# prints the primary metric (selected_root_total_wall_ns).

REPEAT_COUNT="${REPEAT_COUNT:-10}"
BUNDLE_OUT="${BUNDLE_OUT:-target/benchmarks/lazy-eval-run.json}"

mkdir -p "$(dirname "$BUNDLE_OUT")"

cargo run --example benchmark_lazy_eval -- \
  --repeat-count "$REPEAT_COUNT" \
  --bundle-out "$BUNDLE_OUT"

echo ""
echo "=== Primary metric ==="
# Extract selected_root_total_wall_ns from the bundle JSON.
python3 -c "
import json, sys
with open('$BUNDLE_OUT') as f:
    results = json.load(f)
for r in results:
    if r['workload_name'] == 'lazy-selected-root-wide-package-set':
        for m in r['phase_metrics']:
            if m['name'] == 'selected_root_total_wall_ns':
                print(f\"selected_root_total_wall_ns = {m['value']} ns\")
                sys.exit(0)
print('metric not found', file=sys.stderr)
sys.exit(1)
"
