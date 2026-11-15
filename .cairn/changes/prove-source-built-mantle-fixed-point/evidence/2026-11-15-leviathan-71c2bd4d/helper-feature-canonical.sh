#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/helper-source-authority-dev
exec > "$RUN/helper-feature-canonical.log" 2>&1
trap 'status=$?; printf "canonical_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/helper-feature-canonical.exit"; exit "$status"' EXIT
cd "$SOURCE"
export CARGO_TARGET_DIR=/home/brittonr/sbfp-20261113-berdKH/canonical-target-71c2bd4d-fresh-target
export TMPDIR="$RUN/canonical-tmp-71c2bd4d-fresh-target" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash /home/brittonr/sbfp-20261113-berdKH/helper-feature-canonical-inside.sh
/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum src/source_built_fixed_point.rs src/source_built_recipe.rs src/source_built_rust_programs_minicargo_resolve.rs src/source_built_rust_programs_minicargo_resolve_tests.rs
printf '%s\n' 'Ordinary canonical command evidence only; warmed target artifacts are not accepted as a frozen proof runner.'
