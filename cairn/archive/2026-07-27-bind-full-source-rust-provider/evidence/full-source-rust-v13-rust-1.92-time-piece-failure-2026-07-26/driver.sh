#!/bin/sh
set -u

ROOT=/home/brittonr/git/OnixResearch/mantle
RUN_ROOT=/home/brittonr/.cargo-target/mantle-full-source-rust-detached-v13-20260726
TMP_ROOT=$RUN_ROOT/tmp
STATUS_FILE=$RUN_ROOT/status
STATUS_TEMP=$RUN_ROOT/status.tmp
LOG_FILE=$RUN_ROOT/driver.log
MANTLE=/home/brittonr/.cargo-target/debug/mantle
SCRATCH=/home/brittonr/.cargo-target/mantle-full-source-rust-scratch-detached-v13-20260726
OUTPUT=/home/brittonr/.cargo-target/mantle-full-source-rust-provider-detached-v13-20260726
SMOKE=$ROOT/cairn/changes/bind-full-source-rust-provider/evidence/full-source-rust-provider-smoke-v11-2026-07-26
ADMISSION=$ROOT/cairn/changes/bind-full-source-rust-provider/evidence/full-source-provider-admission-v7-rust-host-tools-2026-07-26.json
HOST_TOOLS=$ROOT/cairn/changes/bind-full-source-rust-provider/evidence/full-source-rust-host-tools-v5-linux-headers-2026-07-26/full-source-rust-host-tools.json
RUST_SOURCES=/home/brittonr/.cache/mantle-full-source-20260718/rust-source-archives-20260726
CREATE_NEW_FAILURE=73

for path in "$RUN_ROOT" "$SCRATCH" "$OUTPUT" "$SMOKE"; do
  if [ -e "$path" ]; then
    printf '%s\n' "create-new path already exists: $path" >&2
    exit "$CREATE_NEW_FAILURE"
  fi
done
mkdir -p "$RUN_ROOT" "$TMP_ROOT"
cd "$ROOT" || exit 1

if env TMPDIR="$TMP_ROOT" TMP="$TMP_ROOT" TEMP="$TMP_ROOT" TEMPDIR="$TMP_ROOT" \
  "$MANTLE" --verbose --log-level info bootstrap rust-source-provider \
    --recipe bootstrap/rust-source.ncl \
    --route-plan bootstrap/rust-source-musl-host-plan.ncl \
    --full-source-admission "$ADMISSION" \
    --full-source-host-tools "$HOST_TOOLS" \
    --full-source-rust-sources "$RUST_SOURCES" \
    --scratch-dir "$SCRATCH" \
    --output-dir "$OUTPUT" \
    --smoke \
    --smoke-evidence-dir "$SMOKE" \
    >"$LOG_FILE" 2>&1
then
  status=0
else
  status=$?
fi
printf '%s\n' "$status" >"$STATUS_TEMP"
mv "$STATUS_TEMP" "$STATUS_FILE"
exit "$status"
