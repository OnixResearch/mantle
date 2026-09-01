#!/bin/sh
set -eu
HOST=leviathan.cymric-daggertooth.ts.net
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
LAUNCH="$RUN/authority/source-profile-v84-detached-launch-status.txt"
STATUS="$RUN/authority/source-profile-v84-status.txt"
REFRESH="$RUN/authority/source-profile-v84-refresh.log"
VERIFY="$RUN/authority/source-profile-v84-verification.log"
POLL_ATTEMPTS_MAX=720
POLL_SECONDS=5
launch=$(ssh "$HOST" "cat '$LAUNCH'")
pid=$(printf '%s\n' "$launch" | sed -n 's/^wrapper_pid=//p')
start_ticks=$(printf '%s\n' "$launch" | sed -n 's/^proc_start_ticks=//p')
test -n "$pid"
test -n "$start_ticks"
attempt=0
while test "$attempt" -lt "$POLL_ATTEMPTS_MAX"; do
  attempt=$((attempt + 1))
  observation=$(ssh "$HOST" "if kill -0 '$pid' 2>/dev/null; then actual=\$(cut -d ' ' -f 22 '/proc/$pid/stat'); echo state=running; echo actual_start_ticks=\$actual; else echo state=stopped; fi; test -f '$STATUS' && cat '$STATUS'")
  state=$(printf '%s\n' "$observation" | sed -n 's/^state=//p' | head -1)
  if test "$state" = running; then
    actual=$(printf '%s\n' "$observation" | sed -n 's/^actual_start_ticks=//p' | head -1)
    test "$actual" = "$start_ticks"
    sleep "$POLL_SECONDS"
    continue
  fi
  test "$state" = stopped
  printf '%s\n' "$observation"
  ssh "$HOST" "echo '=== REFRESH ==='; cat '$REFRESH' 2>/dev/null || true; echo '=== VERIFY ==='; cat '$VERIFY' 2>/dev/null || true"
  refresh_exit=$(printf '%s\n' "$observation" | sed -n 's/^refresh_exit_code=//p' | tail -1)
  verify_exit=$(printf '%s\n' "$observation" | sed -n 's/^verify_exit_code=//p' | tail -1)
  test "$refresh_exit" = 0
  test "$verify_exit" = 0
  exit 0
done
echo 'source profile wrapper exceeded watcher budget' >&2
exit 1
