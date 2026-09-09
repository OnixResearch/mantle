#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch_root="$(mktemp -d)"
trap 'rm -rf "$scratch_root"' EXIT

cd "$repo_root"

cargo run -q -p mantle --bin mantle -- \
  __operator-contract --mode raw-descriptors > "$scratch_root/descriptors.json"
cmp config/operator-command-descriptors.json "$scratch_root/descriptors.json"

cargo run -q -p mantle --bin generate-operator-command-contract -- --self-test
nickel typecheck config/operator-surfaces.ncl
nickel export --format json config/operator-surfaces.ncl > "$scratch_root/operator-surfaces.json"
cmp config/operator-surfaces.json "$scratch_root/operator-surfaces.json"

cargo run -q -p mantle --bin mantle -- __operator-contract --mode check
