#!/bin/sh
set -eu

HOST=leviathan.cymric-daggertooth.ts.net
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
LAUNCH_STATUS="$RUN/source-built-fixed-point-v94-unavailable-tools-detached-launch-status.txt"
WRAPPER_STATUS="$RUN/source-built-fixed-point-v94-unavailable-tools-wrapper-status.txt"
LOG="$RUN/source-built-fixed-point-v94-unavailable-tools.log"
OUT="$RUN/source-built-fixed-point-v94-unavailable-tools-20260831"
POLL_ATTEMPTS_MAX=8640
POLL_SECONDS=10
SSH_RETRIES_MAX=60
PROGRESS_INTERVAL=30
SSH_TRANSIENT_STATUS=255

launch=$(ssh "$HOST" "cat '$LAUNCH_STATUS'")
pid=$(printf '%s\n' "$launch" | sed -n 's/^wrapper_pid=//p')
start_ticks=$(printf '%s\n' "$launch" | sed -n 's/^proc_start_ticks=//p')
test -n "$pid"
test -n "$start_ticks"
attempt=0
ssh_retries=0
while test "$attempt" -lt "$POLL_ATTEMPTS_MAX"; do
  attempt=$((attempt + 1))
  set +e
  observation=$(ssh "$HOST" "if kill -0 '$pid' 2>/dev/null; then actual=\$(cut -d ' ' -f 22 '/proc/$pid/stat'); echo state=running; echo actual_start_ticks=\$actual; else echo state=stopped; fi; test -f '$WRAPPER_STATUS' && cat '$WRAPPER_STATUS'" 2>&1)
  ssh_status=$?
  set -e
  if test "$ssh_status" -eq "$SSH_TRANSIENT_STATUS"; then
    ssh_retries=$((ssh_retries + 1))
    echo "poll=$attempt state=ssh-retry retry=$ssh_retries"
    test "$ssh_retries" -le "$SSH_RETRIES_MAX"
    sleep "$POLL_SECONDS"
    continue
  fi
  test "$ssh_status" -eq 0
  ssh_retries=0
  state=$(printf '%s\n' "$observation" | sed -n 's/^state=//p' | head -1)
  if test "$state" = running; then
    actual=$(printf '%s\n' "$observation" | sed -n 's/^actual_start_ticks=//p' | head -1)
    test "$actual" = "$start_ticks"
    if test $((attempt % PROGRESS_INTERVAL)) -eq 0; then
      ssh "$HOST" "echo poll=$attempt state=running wrapper_pid=$pid proc_start_ticks=$actual; tail -8 '$LOG' 2>/dev/null || true"
    fi
    sleep "$POLL_SECONDS"
    continue
  fi
  test "$state" = stopped
  printf '%s\n' "$observation"
  ssh "$HOST" "echo '=== LOG TAIL ==='; tail -180 '$LOG' 2>/dev/null || true; echo '=== ATTEMPT STATUS ==='; find '$RUN' -maxdepth 2 -path '*v94-unavailable-tools*attempt-status.json' -type f -print -exec cat {} \; 2>/dev/null || true; echo '=== OUTPUT ==='; test -e '$OUT' && find '$OUT' -maxdepth 2 -type f | sort | tail -50 || true"
  exit_code=$(printf '%s\n' "$observation" | sed -n 's/^exit_code=//p' | tail -1)
  test -n "$exit_code"
  exit "$exit_code"
done
echo "proof wrapper exceeded watcher budget" >&2
exit 1
