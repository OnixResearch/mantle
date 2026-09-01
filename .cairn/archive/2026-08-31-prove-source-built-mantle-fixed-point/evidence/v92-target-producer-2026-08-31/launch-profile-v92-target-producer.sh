#!/bin/sh
set -eu

RUN=/home/brittonr/mantle-runs/receipt-fix-v31
SCRIPT=/tmp/run-profile-v92-target-producer.sh
LAUNCH_STATUS="$RUN/authority/source-profile-v92-detached-launch-status.txt"
LAUNCH_LOG="$RUN/authority/source-profile-v92-detached-launcher.log"
LAUNCH_PID="$RUN/authority/source-profile-v92-detached-launcher.pid"
WRAPPER_PID="$RUN/authority/source-profile-v92-wrapper.pid"
WRAPPER_STATUS="$RUN/authority/source-profile-v92-status.txt"
WAIT_ATTEMPTS_MAX=30
WAIT_SECONDS=1

for absent in "$LAUNCH_STATUS" "$LAUNCH_LOG" "$LAUNCH_PID" "$WRAPPER_PID" "$WRAPPER_STATUS"; do test ! -e "$absent"; done
test -x "$SCRIPT"
nohup setsid "$SCRIPT" > "$LAUNCH_LOG" 2>&1 < /dev/null &
setsid_pid=$!
echo "$setsid_pid" > "$LAUNCH_PID"
attempt=0
while test ! -f "$WRAPPER_PID"; do
  attempt=$((attempt + 1))
  if test "$attempt" -ge "$WAIT_ATTEMPTS_MAX"; then cat "$LAUNCH_LOG" >&2; exit 1; fi
  sleep "$WAIT_SECONDS"
done
pid=$(cat "$WRAPPER_PID")
kill -0 "$pid"
start_ticks=$(cut -d ' ' -f 22 "/proc/$pid/stat")
command=$(tr '\0' ' ' < "/proc/$pid/cmdline")
{
  echo "launched_at=$(date -Is)"
  echo "launcher=ssh-nohup-setsid"
  echo "setsid_pid=$setsid_pid"
  echo "wrapper_pid=$pid"
  echo "proc_start_ticks=$start_ticks"
  echo "command=$command"
  echo "script=$SCRIPT"
  echo "launcher_log=$LAUNCH_LOG"
  echo "detached_from_pueue=true"
  echo "detached_from_ssh_stdio=true"
} > "$LAUNCH_STATUS"
cat "$LAUNCH_STATUS"
