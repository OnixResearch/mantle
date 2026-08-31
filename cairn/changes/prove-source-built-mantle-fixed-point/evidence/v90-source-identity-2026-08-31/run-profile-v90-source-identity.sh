#!/bin/sh
set -eu

RUN=/home/brittonr/mantle-runs/receipt-fix-v31
BIN="$RUN/mantle-8346c02d-release"
SOURCE="$RUN/source-8346c02d"
FROM="$RUN/authority/source-built-fixed-point-sources-v89-bf2abe26.json"
TO="$RUN/authority/source-built-fixed-point-sources-v90-8346c02d.json"
REFRESH_LOG="$RUN/authority/source-profile-v90-refresh.log"
VERIFY_LOG="$RUN/authority/source-profile-v90-verification.log"
STATUS="$RUN/authority/source-profile-v90-status.txt"
PID_FILE="$RUN/authority/source-profile-v90-wrapper.pid"
TRANSFER="$RUN/source-transfer-v90.txt"
B3="$RUN/b3sum-1.8.5-operator"
DIGEST_HEX_LENGTH=64
EXPECTED_COMMIT=8346c02dd9e73734a821dad04238fe24f88c006d

for required in "$BIN" "$SOURCE" "$FROM" "$TRANSFER" "$B3"; do test -e "$required"; done
for absent in "$TO" "$REFRESH_LOG" "$VERIFY_LOG" "$STATUS" "$PID_FILE"; do test ! -e "$absent"; done
binary_digest=$(sed -n 's/^binary_blake3=//p' "$TRANSFER")
test ${#binary_digest} -eq "$DIGEST_HEX_LENGTH"
test "$("$B3" --no-names "$BIN")" = "$binary_digest"
grep -q "^commit=$EXPECTED_COMMIT$" "$TRANSFER"
grep -q '^binary_roundtrip_parity=exact$' "$TRANSFER"
grep -q '^rsync_checksum_parity=exact$' "$TRANSFER"
grep -q '^root_anchored_excludes=true$' "$TRANSFER"
{
  echo "started_at=$(date -Is)"
  echo "wrapper_pid=$$"
  echo "source_commit=$EXPECTED_COMMIT"
  echo "orchestrator_blake3=$binary_digest"
  echo "from=$FROM"
  echo "to=$TO"
} > "$STATUS"
echo $$ > "$PID_FILE"
set +e
"$BIN" source bundle refresh-mantle-source --from "$FROM" --mantle-source "$SOURCE" --to "$TO" > "$REFRESH_LOG" 2>&1
refresh_exit=$?
set -e
if test "$refresh_exit" -eq 0; then
  set +e
  "$BIN" source bundle verify --from "$TO" > "$VERIFY_LOG" 2>&1
  verify_exit=$?
  set -e
else
  verify_exit=125
fi
profile_digest=$(sed -n 's/^manifest_blake3=\([^ ]*\).*/\1/p' "$VERIFY_LOG" 2>/dev/null || true)
{
  echo "finished_at=$(date -Is)"
  echo "refresh_exit_code=$refresh_exit"
  echo "verify_exit_code=$verify_exit"
  echo "profile_blake3=$profile_digest"
} >> "$STATUS"
test "$refresh_exit" -eq 0
test "$verify_exit" -eq 0
test ${#profile_digest} -eq "$DIGEST_HEX_LENGTH"
grep -q ' readiness=Ready ' "$VERIFY_LOG"
grep -q ' missing=0 stale=0 unsupported=0 untrusted=0$' "$VERIFY_LOG"
