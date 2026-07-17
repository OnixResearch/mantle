#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-first-party-quality.sh

Run the ordinary first-party quality gate:
  1. package-scoped rustfmt check
  2. first-party strict clippy gate
  3. workspace lib/tests
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "[1/3] rustfmt"
cargo_fmt_first_party
note "[2/3] clippy"
"$SCRIPT_DIR/check-first-party-clippy.sh"
note "[3/3] first-party workspace tests (serialized; vendored members excluded)"
cargo_test_workspace_lib_tests
