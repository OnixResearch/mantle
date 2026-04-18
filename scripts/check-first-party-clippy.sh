#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-first-party-clippy.sh

Run the checked-in first-party strict clippy gate.
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "[clippy] first-party strict gate"
cargo_clippy_first_party
