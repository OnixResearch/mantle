#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./quality-gate-common.sh
source "$SCRIPT_DIR/quality-gate-common.sh"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-release-nix-witness-quality.sh

Run the release Nix cross-builder witness quality rail. This is a heavier
release-evidence rail, not part of the ordinary edit-time first-party gate.
It exercises the release CLI witness path for:
  - bit-exact Nix/Mantle artifact agreement
  - digest mismatch without proof-class promotion
  - required witness fail-closed behavior
  - deterministic proof/source binding
  - missing Mantle proof rejection
EOF
}

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 1
fi

enter_quality_gate_repo_root
note "release Nix cross-builder witness quality"
print_command cargo test --test release_cli release_nix_witness -- --nocapture
cargo test --test release_cli release_nix_witness -- --nocapture
