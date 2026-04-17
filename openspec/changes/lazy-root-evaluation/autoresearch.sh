#!/usr/bin/env bash
# Autoresearch harness for lazy root evaluation.
#
# Primary metric: selected_root_total_wall_ns (lower is better)
# Runs the lazy eval benchmark with median-based repeated sampling.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

REPEAT_COUNT="${REPEAT_COUNT:-10}"
BUNDLE_OUT="${1:-}"

echo "=== lazy-root-eval autoresearch ==="
echo "repeat_count: $REPEAT_COUNT"
echo "fixture: tests/fixtures/wide_package_set.ncl"
echo ""

if [ -n "$BUNDLE_OUT" ]; then
    cargo run --example benchmark_lazy_eval -- \
        --repeat-count "$REPEAT_COUNT" \
        --bundle-out "$BUNDLE_OUT"
else
    cargo run --example benchmark_lazy_eval -- \
        --repeat-count "$REPEAT_COUNT"
fi
