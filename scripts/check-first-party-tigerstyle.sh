#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-first-party-tigerstyle.sh [cargo-tigerstyle args]

Run the repo-pinned tigerstyle consumer check through this flake.
Examples:
  ./scripts/check-first-party-tigerstyle.sh
  ./scripts/check-first-party-tigerstyle.sh -p crunch-eval
  ./scripts/check-first-party-tigerstyle.sh -p crunch-build -- --all-targets
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

cd -- "$REPO_ROOT"
exec nix run .#tigerstyle -- check "$@"
