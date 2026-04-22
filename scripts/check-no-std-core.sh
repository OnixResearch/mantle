#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly WASM_TARGET="wasm32-unknown-unknown"

ACTIVE_TOOLCHAIN=""
RUSTUP_CARGO_PATH=""
RUSTUP_TOOLCHAIN_BIN=""
USING_RUSTUP=0
CARGO_RUNNER=()
RUSTC_RUNNER=()

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

ensure_toolchain_environment() {
  if command -v rustup >/dev/null 2>&1; then
    local active_line
    active_line="$(rustup show active-toolchain 2>/dev/null)" \
      || die "rustup has no active toolchain; activate the repo's rustup-managed toolchain first"
    ACTIVE_TOOLCHAIN="${active_line%% *}"
    if [[ -z "$ACTIVE_TOOLCHAIN" ]]; then
      die "rustup returned an empty active toolchain; activate the repo's rustup-managed toolchain first"
    fi

    RUSTUP_CARGO_PATH="$(rustup which cargo --toolchain "$ACTIVE_TOOLCHAIN" 2>/dev/null)" \
      || die "rustup could not resolve cargo for toolchain $ACTIVE_TOOLCHAIN"
    RUSTUP_TOOLCHAIN_BIN="$(cd -- "$(dirname -- "$RUSTUP_CARGO_PATH")" && pwd)"
    export PATH="$RUSTUP_TOOLCHAIN_BIN:$PATH"
    export RUSTUP_TOOLCHAIN="$ACTIVE_TOOLCHAIN"
    USING_RUSTUP=1
    CARGO_RUNNER=(rustup run "$ACTIVE_TOOLCHAIN" cargo)
    RUSTC_RUNNER=(rustup run "$ACTIVE_TOOLCHAIN" rustc)
    return 0
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    die "cargo not found on PATH; run this validation in the repo's Rust toolchain environment"
  fi
  if ! command -v rustc >/dev/null 2>&1; then
    die "rustc not found on PATH; run this validation in the repo's Rust toolchain environment"
  fi

  ACTIVE_TOOLCHAIN="preinstalled"
  note "rustup not found; using preinstalled cargo/rustc on PATH"
  CARGO_RUNNER=(cargo)
  RUSTC_RUNNER=(rustc)
}

wasm_target_available_without_rustup() {
  local target_libdir
  target_libdir="$("${RUSTC_RUNNER[@]}" --print target-libdir --target "$WASM_TARGET" 2>/dev/null)" || return 1
  [[ -d "$target_libdir" ]] || return 1
  find "$target_libdir" -maxdepth 1 -type f | grep -q .
}

ensure_wasm_target() {
  if (( USING_RUSTUP == 1 )); then
    if rustup target list --installed --toolchain "$ACTIVE_TOOLCHAIN" | grep -qx "$WASM_TARGET"; then
      return 0
    fi
    note "installing missing rustup target for $ACTIVE_TOOLCHAIN: $WASM_TARGET"
    print_command rustup target add --toolchain "$ACTIVE_TOOLCHAIN" "$WASM_TARGET"
    rustup target add --toolchain "$ACTIVE_TOOLCHAIN" "$WASM_TARGET" \
      || die "failed to install required target $WASM_TARGET for toolchain $ACTIVE_TOOLCHAIN"
    rustup target list --installed --toolchain "$ACTIVE_TOOLCHAIN" | grep -qx "$WASM_TARGET" \
      || die "rustup did not report target $WASM_TARGET as installed for toolchain $ACTIVE_TOOLCHAIN"
    return 0
  fi

  if wasm_target_available_without_rustup; then
    return 0
  fi

  die "rustup not found and target $WASM_TARGET is not preinstalled under the active rustc sysroot; install it with rustup target add in a rustup-managed environment"
}

run_step() {
  print_command "$@"
  "$@"
}

run_cargo_step() {
  print_command "${CARGO_RUNNER[@]}" "$@"
  "${CARGO_RUNNER[@]}" "$@"
}

cd -- "$REPO_ROOT"
ensure_toolchain_environment
ensure_wasm_target

note "toolchain: $ACTIVE_TOOLCHAIN"
note "[1/10] openspec validate functional-core"
run_step openspec validate functional-core
note "[2/10] host cargo checks"
run_cargo_step check -p crunch-attestation-core
run_cargo_step check -p crunch-project-core
note "[3/10] wasm cargo checks"
run_cargo_step check -p crunch-attestation-core --target "$WASM_TARGET"
run_cargo_step check -p crunch-project-core --target "$WASM_TARGET"
note "[4/10] core tests"
run_cargo_step test -p crunch-attestation-core
run_cargo_step test -p crunch-project-core
note "[5/10] std adapter tests"
run_cargo_step test -p crunch-attestation shell_adapter_keeps_discovery_outside_core
run_cargo_step test -p crunch-project shell_adapter_keeps_refresh_io_outside_core
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
