#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_JSON="$REPO_ROOT/target/bootstrap-blocker-inventory/current.json"
readonly DEFAULT_MARKDOWN="$REPO_ROOT/target/bootstrap-blocker-inventory/current.md"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-bootstrap-blocker-inventory.sh [--report-only] [--json PATH] [--markdown PATH]

Run the lightweight bootstrap blocker inventory/readiness-drift gate. The gate
scans repository-controlled bootstrap sources and canonical bootstrap specs,
then emits JSON and Markdown reports. In the default enforcement mode it fails
only when a full-source promotion claim is present while blocker markers remain.

Options:
  --report-only     inventory blockers but do not enforce promotion-drift failure
  --json PATH       write JSON report (default: target/bootstrap-blocker-inventory/current.json)
  --markdown PATH   write Markdown report (default: target/bootstrap-blocker-inventory/current.md)
EOF
}

json_path="$DEFAULT_JSON"
markdown_path="$DEFAULT_MARKDOWN"
enforce=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-only)
      enforce=0
      shift
      ;;
    --json)
      json_path="${2:?--json requires a path}"
      shift 2
      ;;
    --markdown)
      markdown_path="${2:?--markdown requires a path}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
done

mkdir -p -- "$(dirname -- "$json_path")" "$(dirname -- "$markdown_path")"
cd "$REPO_ROOT"

cargo_bin="${CRUNCH_NIGHTLY_CARGO:-}"
if [[ -z "$cargo_bin" ]]; then
  if [[ -x "$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo" ]]; then
    cargo_bin="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo"
  else
    cargo_bin="cargo"
  fi
fi

args=(
  -Zscript scripts/check-bootstrap-blocker-inventory.rs
  --json "$json_path"
  --markdown "$markdown_path"
  bootstrap
  openspec/specs/bootstrap/spec.md
)

if [[ "$enforce" == 1 ]]; then
  args+=(--enforce)
fi

"$cargo_bin" "${args[@]}"
