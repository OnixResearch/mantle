set -eu
RUN_DIR=/home/brittonr/.cache/mantle-v4-20260908
export CARGO_TARGET_DIR=/home/brittonr/.cache/mantle-v4-target-20260908
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
export TMPDIR="$RUN_DIR/tmp"
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
CHECK_TIMEOUT_SECONDS=1800
mkdir -p "$TMPDIR"
test -x "$SNIX_BUILD_SANDBOX_SHELL"
FAILURES=0
run_check() {
  NAME=$1
  shift
  printf 'started_at=%s\n' "$(date -Iseconds)" > "$RUN_DIR/$NAME.status.txt"
  printf '%s\n' "$@" > "$RUN_DIR/$NAME.argv.txt"
  if timeout "$CHECK_TIMEOUT_SECONDS" "$@" > "$RUN_DIR/$NAME.log" 2>&1; then
    RESULT=0
  else
    RESULT=$?
    FAILURES=$((FAILURES + 1))
  fi
  printf 'finished_at=%s\nexit_code=%s\n' "$(date -Iseconds)" "$RESULT" >> "$RUN_DIR/$NAME.status.txt"
  printf '%s exit_code=%s\n' "$NAME" "$RESULT"
}
printf 'source_commit=%s\n' "$(git rev-parse HEAD)" > "$RUN_DIR/focused-subject.txt"
rustc -Vv >> "$RUN_DIR/focused-subject.txt"
rustfmt -V >> "$RUN_DIR/focused-subject.txt"
run_check fmt rustfmt --edition 2024 --check src/self_build.rs src/source_bundle.rs src/full_source_provider.rs src/protected_exec_ptrace.rs src/source_built_fixed_point_shell.rs src/stagex_transition.rs
run_check fmt-core cargo fmt --check -p crunch-dev-resume-core
run_check diff-check git diff --check
run_check core cargo test --locked --offline -p crunch-dev-resume-core --all-targets
run_check fixed-point cargo test --locked --offline -p mantle --bin mantle source_built_fixed_point
run_check resume cargo test --locked --offline -p mantle --bin mantle dev_resume
run_check ptrace cargo test --locked --offline -p mantle --bin mantle protected_exec_ptrace
run_check stagex cargo test --locked --offline -p mantle --bin mantle stagex_transition
run_check provider cargo test --locked --offline -p mantle --bin mantle full_source_provider
run_check clippy cargo clippy --locked --offline -p mantle -p crunch-dev-resume-core --all-targets --no-deps -- -D warnings
printf 'failed_checks=%s\n' "$FAILURES" > "$RUN_DIR/focused-result.txt"
test "$FAILURES" -eq 0
