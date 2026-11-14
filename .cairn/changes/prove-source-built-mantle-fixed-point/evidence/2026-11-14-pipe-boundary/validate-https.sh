#!/usr/bin/env bash
set -euo pipefail
ROOT=/home/brittonr/sbfp-20261113-berdKH/pipe-boundary-validation.YinuAM
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$ROOT/validation-https.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "pipe_validation_exit=%s\n" "$status"; printf "%s\n" "$status" > "$ROOT/validation-https.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
test "$("$B3" --no-names "$ROOT/source/src/source_built_mantle_programs_native.rs")" = 5cb0ebc1fa11afd7c28f8381a46791b4e20b57a10ab3b1ad4adfc9f734888403
test "$("$B3" --no-names "$ROOT/source/flake.nix")" = 6997b85fddbdff4ffd543364db73a9b07c34bf3adb4dafdd1db5ee9597c64e7e
test -d "$ROOT/tmp"
cd "$ROOT/source"
export TMPDIR="$ROOT/tmp" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustc -Vv; cargo -V; perl --version; cargo test --release --locked --offline -p mantle --bin mantle source_built_mantle_programs::native::tests:: -- --test-threads=1; cargo fmt --check -p mantle -v; cargo clippy --release --locked --offline -p mantle --bin mantle --tests --no-deps -- -D warnings'
date -Is
finished=1
printf 'pipe_validation_complete=1\n'
