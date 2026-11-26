#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-first-party-quality.sh

Run the ordinary first-party quality gate:
  1. bounded GCC 4.0 configure-bridge contract
  2. checked-in evidence size budget and retention guard
  3. package-scoped rustfmt check
  4. first-party strict clippy gate
  5. workspace lib/tests
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "[1/5] bounded GCC 4.0 configure bridge"
"$SCRIPT_DIR/check-gcc40-configure-bridge.rs" --self-test
note "[2/5] checked-in evidence size budget"
cargo -Zscript "$SCRIPT_DIR/check-evidence-size-budget.rs" --self-test
note "[3/5] rustfmt"
cargo_fmt_first_party
note "[4/5] clippy"
"$SCRIPT_DIR/check-first-party-clippy.sh"
note "[5/5] first-party workspace tests (serialized; vendored members excluded)"
cargo_test_workspace_lib_tests
