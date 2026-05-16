#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-mantle-transcript-quality.sh

Run the fast Mantle executable transcript quality rail. This gate covers the
Markdown transcript parser/runner plus checked fast transcript fixtures without
running heavier release or self-hosting evidence rails.
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "Mantle executable transcript quality"
print_command cargo test --test transcript_cli -- --nocapture
cargo test --test transcript_cli -- --nocapture
