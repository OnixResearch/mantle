#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-release-determinism-quality.sh

Run the release determinism quality rail:
  1. generated deterministic proof smoke
  2. smoke receipt schema/log BLAKE3 validation

This is a heavier release-evidence rail, not part of the ordinary edit-time
first-party gate.
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "[1/2] release determinism smoke"
print_command cargo -Zscript scripts/release-determinism-smoke.rs
cargo -Zscript scripts/release-determinism-smoke.rs

note "[2/2] release determinism smoke receipt validation"
readonly RECEIPT_PATH="target/release-determinism-smoke/latest/receipt.json"
print_command cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs "$RECEIPT_PATH"
cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs "$RECEIPT_PATH"
