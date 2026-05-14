#!/usr/bin/env bash
set -euo pipefail

readonly QUALITY_GATE_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly QUALITY_GATE_REPO_ROOT="$(cd -- "$QUALITY_GATE_SCRIPT_DIR/.." && pwd)"

# Keep first-party package scope in one checked-in place.
readonly -a FIRST_PARTY_PACKAGES=(
  mantle
  crunch-attestation
  crunch-build
  crunch-delta
  crunch-eval
  crunch-glue
  crunch-pipeline
  crunch-project
  crunch-shell
  crunch-store
)

# Keep vendored workspace exclusions in one checked-in place.
readonly -a VENDORED_WORKSPACE_EXCLUDES=(
  fuse-backend-rs
  nix-compat
  nix-compat-derive
  snix-build
  snix-castore
  snix-store
  snix-tracing
)

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*" >&2
}

print_command() {
  local arg

  printf 'running:' >&2
  for arg in "$@"; do
    printf ' %q' "$arg" >&2
  done
  printf '\n' >&2
}

require_quality_gate_repo_root() {
  if [[ ! -f "$QUALITY_GATE_REPO_ROOT/Cargo.toml" ]]; then
    die "expected Cargo.toml at repo root: $QUALITY_GATE_REPO_ROOT"
  fi

  if [[ ! -f "$QUALITY_GATE_REPO_ROOT/rust-toolchain.toml" ]]; then
    die "expected rust-toolchain.toml at repo root: $QUALITY_GATE_REPO_ROOT"
  fi
}

enter_quality_gate_repo_root() {
  require_quality_gate_repo_root
  cd -- "$QUALITY_GATE_REPO_ROOT"
}

cargo_fmt_first_party() {
  local -a args=(fmt --check)
  local package_name

  for package_name in "${FIRST_PARTY_PACKAGES[@]}"; do
    args+=(-p "$package_name")
  done

  print_command cargo "${args[@]}"
  cargo "${args[@]}"
}

cargo_clippy_first_party() {
  local -a args=(clippy --workspace --all-targets --no-deps)
  local package_name

  # --no-deps suppresses dependency lint output.
  # --exclude removes vendored workspace-member targets from the strict gate.
  for package_name in "${VENDORED_WORKSPACE_EXCLUDES[@]}"; do
    args+=(--exclude "$package_name")
  done
  args+=(-- -D warnings)

  print_command cargo "${args[@]}"
  cargo "${args[@]}"
}

cargo_test_workspace_lib_tests() {
  local -a args=(test --workspace --lib --tests)

  print_command cargo "${args[@]}"
  cargo "${args[@]}"
}
