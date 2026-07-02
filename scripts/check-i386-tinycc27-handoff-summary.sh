#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly RUNNER_DIR="${TMPDIR:-$REPO_ROOT/target/i386-handoff-summary}/i386-handoff-summary-runner"
readonly RUNNER_BIN="$RUNNER_DIR/check-i386-tinycc27-handoff-summary"

mkdir -p -- "$RUNNER_DIR"
cd "$REPO_ROOT"

rustc_bin="${CRUNCH_NIGHTLY_RUSTC:-}"
if [[ -z "$rustc_bin" ]]; then
  if [[ -x "$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc" ]]; then
    rustc_bin="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc"
  else
    rustc_bin="rustc"
  fi
fi

rustc_args=(
  --edition=2024
  scripts/check-i386-tinycc27-handoff-summary.rs
  -o "$RUNNER_BIN"
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
"$RUNNER_BIN" "$@"
