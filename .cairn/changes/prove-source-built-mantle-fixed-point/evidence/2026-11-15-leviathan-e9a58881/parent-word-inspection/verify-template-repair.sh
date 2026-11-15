#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
SOURCE=/home/brittonr/sbfp-20261113-berdKH/repair-rust-template-admission-source
exec > "$RUN/verify-template-repair.log" 2>&1
trap 'status=$?; printf "verification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/verify-template-repair.exit"; exit "$status"' EXIT
cd "$SOURCE"
export TMPDIR="$RUN/runner-tmp-e9a58881-typed-index" XDG_CACHE_HOME="$RUN/tooling-cache" CARGO_TARGET_DIR="$RUN/runner-build-target" CARGO_BUILD_JOBS=8 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTC_BOOTSTRAP
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=url.https://github.com/.insteadOf GIT_CONFIG_VALUE_0=ssh://git@github.com/ GIT_TERMINAL_PROMPT=0
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c 'set -euo pipefail; rustfmt --edition 2024 --config skip_children=true src/source_built_rust_programs_minicargo.rs src/source_built_rust_programs_successors.rs src/source_built_rust_programs_successors_profile.rs src/source_built_rust_programs_successors_profile_tests.rs; cargo test --release --locked --offline -p mantle --bin mantle original_rpath_profile_survives_strict_deferred_invocation_admission -- --test-threads=1; cargo test --release --locked --offline -p mantle --bin mantle source_built_recipe::tests::invocation_cannot_invent_paths_names_control_reads_or_unused_dependencies -- --test-threads=1; cargo test --release --locked --offline -p mantle --bin mantle source_built_rust_resolve::tests:: -- --test-threads=1'
