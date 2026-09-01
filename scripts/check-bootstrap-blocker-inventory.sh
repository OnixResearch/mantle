#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_JSON="$REPO_ROOT/target/bootstrap-blocker-inventory/current.json"
readonly DEFAULT_MARKDOWN="$REPO_ROOT/target/bootstrap-blocker-inventory/current.md"

usage() {
  cat <<'EOF'
Usage: ./scripts/check-bootstrap-blocker-inventory.sh [--report-only] [--self-test] [--json PATH] [--markdown PATH]

Run the lightweight bootstrap blocker inventory/readiness-drift gate. The gate
scans repository-controlled bootstrap sources and canonical bootstrap specs,
then emits JSON and Markdown reports. Report-only mode exits successfully after
valid report generation even when blockers remain. Default enforcement mode
fails unless the checked baseline stays clean: 0 unsuppressed actionable findings
and 0 promotion claims.

Options:
  --report-only     write inventory reports without clean-baseline enforcement
  --self-test       run built-in matcher regression tests before scanning
  --json PATH       write JSON report (default: target/bootstrap-blocker-inventory/current.json)
  --markdown PATH   write Markdown report (default: target/bootstrap-blocker-inventory/current.md)
EOF
}

json_path="$DEFAULT_JSON"
markdown_path="$DEFAULT_MARKDOWN"
enforce=1
self_test=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-only)
      enforce=0
      shift
      ;;
    --self-test)
      self_test=1
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

rustc_bin="${CRUNCH_NIGHTLY_RUSTC:-}"
if [[ -z "$rustc_bin" && -x "${cargo_bin%/cargo}/rustc" ]]; then
  rustc_bin="${cargo_bin%/cargo}/rustc"
fi
if [[ -z "$rustc_bin" ]]; then
  if [[ -x "$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc" ]]; then
    rustc_bin="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc"
  else
    rustc_bin="rustc"
  fi
fi

runner_dir="${TMPDIR:-$REPO_ROOT/target/bootstrap-blocker-inventory}/bootstrap-blocker-inventory-runner"
runner_bin="$runner_dir/check-bootstrap-blocker-inventory"
mkdir -p -- "$runner_dir"

rustc_args=(
  --edition=2024
  scripts/check-bootstrap-blocker-inventory.rs
  -o "$runner_bin"
)
if [[ -n "${CC:-}" ]]; then
  rustc_args+=(-C "linker=$CC")
else
  for linker_candidate in \
    /nix/store/*-clang-wrapper-*/bin/clang \
    /run/current-system/sw/bin/cc \
    /usr/bin/cc
  do
    if [[ -x "$linker_candidate" ]]; then
      rustc_args+=(-C "linker=$linker_candidate")
      break
    fi
  done
fi
"$rustc_bin" "${rustc_args[@]}"

args=(
  --json "$json_path"
  --markdown "$markdown_path"
  bootstrap
  openspec/specs/bootstrap/spec.md
)

if [[ "$enforce" == 1 ]]; then
  args+=(--enforce --require-clean)
fi
if [[ "$self_test" == 1 ]]; then
  args+=(--self-test)
fi

b3sum_bin="$(command -v b3sum || true)"
if [[ -z "$b3sum_bin" || "$b3sum_bin" != /* || ! -x "$b3sum_bin" ]]; then
  echo "error: absolute executable b3sum is required for proof-bound blocker classification" >&2
  exit 2
fi
export MANTLE_BOOTSTRAP_BLOCKER_B3SUM="$b3sum_bin"

"$runner_bin" "${args[@]}"
