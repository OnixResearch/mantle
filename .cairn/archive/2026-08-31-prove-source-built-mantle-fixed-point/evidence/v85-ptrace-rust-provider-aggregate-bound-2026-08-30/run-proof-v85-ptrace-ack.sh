#!/bin/sh
set -eu

RUN=/home/brittonr/mantle-runs/receipt-fix-v31
BIN="$RUN/mantle-dc96b465-release"
PROFILE="$RUN/authority/source-built-fixed-point-sources-v85-dc96b465.json"
VERIFY="$RUN/authority/source-profile-v85-verification.log"
TRANSFER="$RUN/source-transfer-v85.txt"
CHECKPOINT_STORE="$RUN/source-built-proof-checkpoints"
NATIVE_ATTEMPT="$RUN/source-built-native-prefix-v61-repaired-20260828"
OUT="$RUN/source-built-fixed-point-v85-ptrace-acknowledged-20260830"
LOG="$RUN/source-built-fixed-point-v85-ptrace-acknowledged.log"
STATUS="$RUN/source-built-fixed-point-v85-ptrace-acknowledged-wrapper-status.txt"
PID_FILE="$RUN/source-built-fixed-point-v85-ptrace-acknowledged-wrapper.pid"
HOST_FACTS="$RUN/source-built-fixed-point-v85-ptrace-acknowledged-host-facts.txt"
BWRAP=/nix/store/z0ddzd81dfmh1bf113zdmyj91vvyrjyc-bubblewrap-0.11.2/bin/bwrap
SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
EXPECTED_PROFILE_BLAKE3=464632a236ffdfa3c2d01eb81add6dc857cc408f140acbe7fd3f9478b6d0b9d2
EXPECTED_ORCHESTRATOR_BLAKE3=8f9e89e6b6a8d92f91ba6d0072110ada5199955d9a79db63201a8430e0c0ee09
EXPECTED_STAGEX_LINEAGE_BLAKE3=e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d
EXPECTED_NATIVE_PROVIDER_BLAKE3=63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9
PROOF_DISK_BYTES_MAX=700000000000
PROOF_JOBS=16
DIGEST_HEX_LENGTH=64
EXPECTED_HOST=leviathan
EXPECTED_ARCH=x86_64
EXPECTED_COMMIT=dc96b465b4acfae05042ae4bb356066d372f6656
B3="$RUN/b3sum-1.8.5-operator"

PROFILE_DIGEST=$(sed -n 's/^manifest_blake3=\([^ ]*\).*/\1/p' "$VERIFY")
test ${#PROFILE_DIGEST} -eq "$DIGEST_HEX_LENGTH"
test "$PROFILE_DIGEST" = "$EXPECTED_PROFILE_BLAKE3"
for required in "$BIN" "$BWRAP" "$SANDBOX_SHELL" "$B3" "$PROFILE" "$TRANSFER" "$CHECKPOINT_STORE" "$NATIVE_ATTEMPT"; do test -e "$required"; done
test "$("$B3" --no-names "$BIN")" = "$EXPECTED_ORCHESTRATOR_BLAKE3"
grep -q ' readiness=Ready ' "$VERIFY"
grep -q ' missing=0 stale=0 unsupported=0 untrusted=0$' "$VERIFY"
grep -q "^commit=$EXPECTED_COMMIT$" "$TRANSFER"
grep -q "^binary_blake3=$EXPECTED_ORCHESTRATOR_BLAKE3$" "$TRANSFER"
grep -q '^binary_roundtrip_parity=exact$' "$TRANSFER"
grep -q '^rsync_checksum_parity=exact$' "$TRANSFER"
grep -q '^root_anchored_excludes=true$' "$TRANSFER"
grep -q "^restored_output_blake3=$EXPECTED_NATIVE_PROVIDER_BLAKE3$" "$NATIVE_ATTEMPT/native-prefix-mode-repair-receipt.txt"
test "$(hostname -s)" = "$EXPECTED_HOST"
test "$(uname -m)" = "$EXPECTED_ARCH"
for absent in "$OUT" "$PID_FILE" "$LOG" "$STATUS" "$HOST_FACTS"; do test ! -e "$absent"; done
free_blocks=$(stat -f -c %a "$RUN")
block_size=$(stat -f -c %S "$RUN")
free_bytes=$((free_blocks * block_size))
test "$free_bytes" -ge "$PROOF_DISK_BYTES_MAX"
{
  echo "captured_at=$(date -Is)"
  echo "host=$(hostname -f)"
  echo "system=$(uname -s)"
  echo "architecture=$(uname -m)"
  echo "kernel=$(uname -r)"
  echo "processors=$(getconf _NPROCESSORS_ONLN)"
  echo "open_file_soft_limit=$(ulimit -Sn)"
  echo "open_file_hard_limit=$(ulimit -Hn)"
  echo "free_bytes_before=$free_bytes"
  echo "proof_disk_bytes_max=$PROOF_DISK_BYTES_MAX"
  echo "proof_jobs=$PROOF_JOBS"
  echo "bwrap=$BWRAP"
  echo "sandbox_shell=$SANDBOX_SHELL"
  echo "source_commit=$EXPECTED_COMMIT"
  echo "source_transfer_parity=exact"
  echo "profile_blake3=$PROFILE_DIGEST"
  echo "orchestrator_blake3=$EXPECTED_ORCHESTRATOR_BLAKE3"
  echo "protected_exec_mechanism=seccomp-ret-trace-ptrace-v1-acknowledged-root"
  echo "checkpoint_mode=native-prefix-isolated"
  echo "checkpoint_store=$CHECKPOINT_STORE"
  echo "native_checkpoint_attempt=$NATIVE_ATTEMPT"
} > "$HOST_FACTS"
{
  echo "started_at=$(date -Is)"
  echo "wrapper_pid=$$"
  echo "source_commit=$EXPECTED_COMMIT"
  echo "source_transfer_parity=exact"
  echo "profile_blake3=$PROFILE_DIGEST"
  echo "orchestrator_blake3=$EXPECTED_ORCHESTRATOR_BLAKE3"
  echo "protected_exec_mechanism=seccomp-ret-trace-ptrace-v1-acknowledged-root"
  echo "checkpoint_mode=native-prefix-isolated"
  echo "native_checkpoint_attempt=$NATIVE_ATTEMPT"
  echo "output=$OUT"
  echo "proof_disk_bytes_max=$PROOF_DISK_BYTES_MAX"
  echo "proof_jobs=$PROOF_JOBS"
} > "$STATUS"
echo $$ > "$PID_FILE"
export CRUNCH_NO_FUSE=1
export SNIX_BUILD_SANDBOX_SHELL="$SANDBOX_SHELL"
export PATH="$(dirname "$BWRAP"):$PATH"
set +e
"$BIN" self-build --source-built-fixed-point --source-profile "$PROFILE" --expected-source-profile-blake3 "$PROFILE_DIGEST" --expected-stagex-lineage-blake3 "$EXPECTED_STAGEX_LINEAGE_BLAKE3" --expected-native-provider-blake3 "$EXPECTED_NATIVE_PROVIDER_BLAKE3" --proof-bwrap "$BWRAP" --proof-sandbox-shell "$SANDBOX_SHELL" --proof-disk-bytes-max "$PROOF_DISK_BYTES_MAX" --proof-checkpoint-store "$CHECKPOINT_STORE" --proof-native-checkpoint-attempt "$NATIVE_ATTEMPT" --out "$OUT" --jobs "$PROOF_JOBS" --strict-hermetic --no-substitute --verbose --log-level info > "$LOG" 2>&1
exit_code=$?
set -e
{
  echo "finished_at=$(date -Is)"
  echo "exit_code=$exit_code"
} >> "$STATUS"
exit "$exit_code"
