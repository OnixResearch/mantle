#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly WASM_TARGET="wasm32-unknown-unknown"

note() {
  printf '%s\n' "$*" >&2
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

print_command() {
  local arg
  printf 'running:' >&2
  for arg in "$@"; do
    printf ' %q' "$arg" >&2
  done
  printf '\n' >&2
}

ensure_rustup_target() {
  if rustc --print target-libdir --target "$WASM_TARGET" >/dev/null 2>&1; then
    return 0
  fi
  if ! command -v rustup >/dev/null 2>&1; then
    die "rustup not found on PATH and target $WASM_TARGET is unavailable"
  fi
  if rustup target list --installed | grep -qx "$WASM_TARGET"; then
    return 0
  fi
  note "installing missing rustup target: $WASM_TARGET"
  print_command rustup target add "$WASM_TARGET"
  rustup target add "$WASM_TARGET" || die "failed to install required target $WASM_TARGET"
}

run_step() {
  print_command "$@"
  "$@"
}

cd -- "$REPO_ROOT"
ensure_rustup_target

note "[1/10] openspec validate"
run_step openspec validate no-std-functional-core
note "[2/10] host cargo checks"
run_step cargo check -p crunch-attestation-core
run_step cargo check -p crunch-project-core
note "[3/10] wasm cargo checks"
run_step cargo check -p crunch-attestation-core --target "$WASM_TARGET"
run_step cargo check -p crunch-project-core --target "$WASM_TARGET"
note "[4/10] core tests"
run_step cargo test -p crunch-attestation-core
run_step cargo test -p crunch-project-core
note "[5/10] std adapter tests"
run_step cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core
run_step cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core
note "[6/10] dependency boundary"
run_step ./scripts/check-no-std-core-deps.sh
note "[7/10] purity"
run_step ./scripts/check-no-std-core-purity.sh
note "[8/10] scope"
run_step ./scripts/check-no-std-core-scope.sh
note "[9/10] API shape"
run_step ./scripts/check-no-std-core-api-shape.sh
note "[10/10] ownership"
run_step ./scripts/check-no-std-core-ownership.sh
