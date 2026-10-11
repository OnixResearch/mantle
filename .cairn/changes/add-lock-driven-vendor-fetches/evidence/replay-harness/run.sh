#!/usr/bin/env bash
# Run detached by parent after current release binary is complete. Pass optional
# --scratch and --store-root paths under the rust-efficiency cohort; no cleanup.
set -euo pipefail
here=$(dirname "$(readlink -f "$0")")
exec python3 "$here/replay.py" "$@"
