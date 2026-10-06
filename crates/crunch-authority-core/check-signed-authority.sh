#!/usr/bin/env bash
set -euo pipefail

manifest_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
exec cargo test --manifest-path "$manifest_dir/Cargo.toml" --lib --tests -- --test-threads 1
