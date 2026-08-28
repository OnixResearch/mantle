#!/bin/sh
set -eu
RUN=/home/brittonr/mantle-runs/receipt-fix-v31
ORIGIN="$RUN/.source-built-fixed-point-v61-runtime-shim-prefix-cold-20260825.source-built-fixed-point-staging-3637585"
FINAL="$RUN/source-built-native-prefix-v61-repaired-20260828"
STAGING="$RUN/.source-built-native-prefix-v61-repaired-20260828.tmp"
PROVIDER_BASENAME=kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain
PROVIDER="$STAGING/native-store/$PROVIDER_BASENAME"
SOURCE_CLOSURE="$STAGING/native-source-closure.json"
REPORT="$STAGING/repaired-native-provider-admission.json"
RECEIPT="$STAGING/native-prefix-mode-repair-receipt.txt"
BIN="$RUN/mantle-b36c782a-release"
EXPECTED_OUTPUT_BLAKE3=63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9
MUTATED_OUTPUT_BLAKE3=a5c6a9749a93dcea577784a8eace2eda6e0f7a4cf1889140ec9255d10336a118
EXPECTED_SOURCE_CLOSURE_BLAKE3=7a97f836c8daa3dc5cbdf51a2d6ca59e7b995debac22fe5d08c091e667828778
READ_ONLY_MODE=444
EXPECTED_REPAIR_COUNT=6
for required in "$ORIGIN" "$BIN"; do test -e "$required"; test ! -L "$required"; done
for absent in "$FINAL" "$STAGING"; do test ! -e "$absent"; done
mkdir -m 700 "$STAGING"
for name in attempt-status.json source-built-fixed-point-plan.json full-source-provider-admission.json native-provider-action-plan.json native-provider-action-reconciliation.json native-source-closure.json; do cp -a "$ORIGIN/$name" "$STAGING/$name"; done
for name in full-source-rust-host-tools transcripts native-store stagex-transition-execution; do cp -a "$ORIGIN/$name" "$STAGING/$name"; done
set -- \
  "$PROVIDER/x86_64-linux-musl/lib/Scrt1.o" \
  "$PROVIDER/x86_64-linux-musl/lib/crt1.o" \
  "$PROVIDER/x86_64-linux-musl/lib/crti.o" \
  "$PROVIDER/x86_64-linux-musl/lib/crtn.o" \
  "$PROVIDER/x86_64-linux-musl/lib/libc.a" \
  "$PROVIDER/x86_64-linux-musl/lib/rcrt1.o"
test "$#" -eq "$EXPECTED_REPAIR_COUNT"
for path in "$@"; do test -f "$path"; test ! -L "$path"; test -w "$path"; chmod "$READ_ONLY_MODE" "$path"; test ! -w "$path"; done
remaining_writable=$(find "$PROVIDER" -xdev -type f -perm -u+w -print | wc -l)
test "$remaining_writable" -eq 0
"$BIN" --json bootstrap full-source-provider-admit \
  --provider-dir "$PROVIDER" \
  --expected-output-blake3 "$EXPECTED_OUTPUT_BLAKE3" \
  --source-closure "$SOURCE_CLOSURE" \
  --expected-source-closure-blake3 "$EXPECTED_SOURCE_CLOSURE_BLAKE3" \
  --report "$REPORT" > "$STAGING/repaired-native-provider-admission.stdout"
grep -q "\"output_digest_blake3\": \"$EXPECTED_OUTPUT_BLAKE3\"" "$REPORT"
{
  echo "derived_at=$(date -Is)"
  echo "origin=$ORIGIN"
  echo "origin_observed_mutated_blake3=$MUTATED_OUTPUT_BLAKE3"
  echo "derived=$FINAL"
  echo "restored_output_blake3=$EXPECTED_OUTPUT_BLAKE3"
  echo "source_closure_blake3=$EXPECTED_SOURCE_CLOSURE_BLAKE3"
  echo "repaired_file_count=$EXPECTED_REPAIR_COUNT"
  echo "repair=remove-owner-write-from-exact-musl-crt-and-libc-archive-set"
  echo "copy_semantics=ordinary-byte-copy"
  echo "verdict=derived-prefix-restores-previously-admitted-provider-identity"
} > "$RECEIPT"
mv "$STAGING" "$FINAL"
test -d "$FINAL"
test ! -L "$FINAL"
cat "$FINAL/native-prefix-mode-repair-receipt.txt"
