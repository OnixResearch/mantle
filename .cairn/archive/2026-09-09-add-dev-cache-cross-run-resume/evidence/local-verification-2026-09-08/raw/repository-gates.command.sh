set -eu
RUN_DIR=/home/brittonr/.cache/mantle-v4-20260908
CHANGE=add-dev-cache-cross-run-resume
CAIRN=/nix/store/zddawm6d27cgvic58y2jipgnqfh395n0-cairn-0.1.0/bin/cairn
NIX_MAX_JOBS=2
NIX_BUILD_CORES=4
GATE_TIMEOUT_SECONDS=1800
FAILURES=0
run_check() {
  NAME=$1
  shift
  printf 'started_at=%s\n' "$(date -Iseconds)" > "$RUN_DIR/$NAME.status.txt"
  printf '%s\n' "$@" > "$RUN_DIR/$NAME.argv.txt"
  if timeout "$GATE_TIMEOUT_SECONDS" "$@" > "$RUN_DIR/$NAME.log" 2>&1; then RESULT=0; else RESULT=$?; FAILURES=$((FAILURES + 1)); fi
  printf 'finished_at=%s\nexit_code=%s\n' "$(date -Iseconds)" "$RESULT" >> "$RUN_DIR/$NAME.status.txt"
  printf '%s exit_code=%s\n' "$NAME" "$RESULT"
}
printf 'source_commit=%s\npolicy_tool=%s\n' "$(git rev-parse HEAD)" "$CAIRN" > "$RUN_DIR/repository-subject.txt"
git status --short --branch >> "$RUN_DIR/repository-subject.txt"
run_check cairn-validate "$CAIRN" validate --root .
run_check tracey "$CAIRN" tracey coverage --root .
run_check gate-proposal "$CAIRN" gate proposal "$CHANGE" --root .
run_check gate-design "$CAIRN" gate design "$CHANGE" --root .
run_check gate-tasks "$CAIRN" gate tasks "$CHANGE" --root .
run_check flake-eval nix flake check --no-build --no-eval-cache
for CHECK in dev-resume-architecture dev-resume-core dev-resume-core-wasm dev-resume-stage-publication dev-resume-integration; do
  run_check "nix-$CHECK" nix build --no-link -L --max-jobs "$NIX_MAX_JOBS" --cores "$NIX_BUILD_CORES" ".#checks.x86_64-linux.$CHECK"
done
printf 'failed_checks=%s\n' "$FAILURES" > "$RUN_DIR/repository-result.txt"
test "$FAILURES" -eq 0
