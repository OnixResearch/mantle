#!/bin/sh
set -eu
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
CHAIN="$RUN/.source-built-fixed-point-v82-seccomp-response-bound-20260829.source-built-fixed-point-staging-1107472/rust-provider-scratch/rustc-stage1-chain"
ORIG="$CHAIN/rust-1.93.1-stage1"
DIAG="$CHAIN/rust-1.93.1-stage1-diag"
STRACE_LOG="$DIAG/execve.strace"
RUN_LOG="$DIAG/diag-run.log"
STATUS="$DIAG/diag-status.txt"
for absent in "$DIAG" "$STRACE_LOG" "$RUN_LOG" "$STATUS"; do test ! -e "$absent"; done
test -d "$ORIG"
mkdir "$DIAG"
{
  echo "started_at=$(date -Is)"
  echo "wrapper_pid=$$"
  echo "origin=$ORIG"
  echo "diagnostic_root=$DIAG"
  echo "seccomp=disabled-diagnostic-only"
  echo "strace=strace -f -qq -e trace=execve"
} > "$STATUS"
cp -a "$ORIG/rustc-stage1-sources" "$DIAG/rustc-stage1-sources"
cp -a "$ORIG/run-rustc-stage1.sh" "$DIAG/run-rustc-stage1.sh"
sed -i 's#rust-1.93.1-stage1#rust-1.93.1-stage1-diag#g' "$DIAG/run-rustc-stage1.sh"
grep -q 'rust-1.93.1-stage1-diag' "$DIAG/run-rustc-stage1.sh"
grep -q 'rust-1.92.0-stage1/rustc-stage1-provider-candidate' "$DIAG/run-rustc-stage1.sh"
{
  echo "copied_at=$(date -Is)"
  echo "path_rewrite=origin-dir-to-diag-dir-only"
} >> "$STATUS"
set +e
strace -f -qq -e trace=execve -o "$STRACE_LOG" sh "$DIAG/run-rustc-stage1.sh" > "$RUN_LOG" 2>&1
exit_code=$?
set -e
{
  echo "finished_at=$(date -Is)"
  echo "exit_code=$exit_code"
} >> "$STATUS"
gzip -f "$STRACE_LOG"
exit "$exit_code"
