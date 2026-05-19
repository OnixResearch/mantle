#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-bootstrap-parity-snapshot.sh

Run the bootstrap parity snapshot rail:
  1. write target/bootstrap-parity-snapshot/latest/{report.json,receipt.json,stderr.log}
  2. validate the receipt against the saved report BLAKE3 and axis/row summary

This is an operator evidence rail. It records current parity blockers and does
not claim live-bootstrap, Guix, or StageX parity completion.
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root

# The user's global Cargo config may route rustc through a stale sccache wrapper
# and shared ~/.cargo-target. Force a local, wrapper-free run for inspectable
# bootstrap parity evidence.
export CARGO_BUILD_RUSTC_WRAPPER=
export CARGO_TARGET_DIR="$PWD/target"
export TMPDIR="${TMPDIR:-/tmp}"
unset RUSTC_WRAPPER SCCACHE_DIR SCCACHE_CACHE_SIZE SCCACHE_ERROR_LOG SCCACHE_IDLE_TIMEOUT

note "[1/2] bootstrap parity snapshot"
print_command cargo -Zscript scripts/bootstrap-parity-snapshot.rs
cargo -Zscript scripts/bootstrap-parity-snapshot.rs

note "[2/2] bootstrap parity snapshot receipt validation"
readonly RECEIPT_PATH="target/bootstrap-parity-snapshot/latest/receipt.json"
print_command cargo -Zscript scripts/check-bootstrap-parity-snapshot-receipt.rs "$RECEIPT_PATH"
cargo -Zscript scripts/check-bootstrap-parity-snapshot-receipt.rs "$RECEIPT_PATH"
