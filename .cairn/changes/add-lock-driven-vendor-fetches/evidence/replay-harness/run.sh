#!/usr/bin/env bash
# Run detached by parent after current release binary is complete. Pass optional
# --scratch and --store-root paths under the rust-efficiency cohort; no cleanup.
set -euo pipefail
here=$(dirname "$(readlink -f "$0")")
python=/nix/store/3n4qphl9s728sz8frmpqqrv9b1m87g68-python3-3.14.7/bin/python3
test -x "$python"
export PATH="${python%/bin/python3}/bin:$PATH"
exec "$python" "$here/replay.py" "$@"
