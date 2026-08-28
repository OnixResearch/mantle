#!/bin/sh
set -eu
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
ROOT="$RUN/.source-built-fixed-point-v78-native-prefix-resume-20260827.source-built-fixed-point-staging-592125"
RECEIPT="$RUN/prune-v78-failed-root-for-v79.txt"
PRESERVED_EVIDENCE_COMMIT=b36c782a
RETAINED_NATIVE_PREFIX=v61
test -d "$RUN"
test ! -L "$RUN"
test -d "$ROOT"
test ! -L "$ROOT"
test ! -e "$RECEIPT"
grep -q '"status": "failed"' "$ROOT/attempt-status.json"
before=$(($(stat -f -c %a "$RUN") * $(stat -f -c %S "$RUN")))
size=$(du -sb "$ROOT" 2>/dev/null | cut -f1)
{
  echo "pruned_at=$(date -Is)"
  echo "root=$ROOT"
  echo "status=failed"
  echo "preserved_evidence_commit=$PRESERVED_EVIDENCE_COMMIT"
  echo "size_bytes=$size"
  echo "free_bytes_before=$before"
} > "$RECEIPT"
chmod -R u+w "$ROOT" 2>/dev/null || true
rm -rf -- "$ROOT"
test ! -e "$ROOT"
after=$(($(stat -f -c %a "$RUN") * $(stat -f -c %S "$RUN")))
{
  echo "free_bytes_after=$after"
  echo "reclaimed_bytes=$((after-before))"
  echo "retained_native_prefix=$RETAINED_NATIVE_PREFIX"
  echo "verdict=failed-root-pruned-after-local-evidence-preservation"
} >> "$RECEIPT"
cat "$RECEIPT"
