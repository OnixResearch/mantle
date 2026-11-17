#!/usr/bin/env bash
set -euo pipefail
cd '/home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources'
test "$#" -eq 2
PLAN_BLAKE3="$1"
RECEIPT_BYTES_BLAKE3="$2"
[[ "$PLAN_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
[[ "$RECEIPT_BYTES_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
COLD='/home/brittonr/sbfp-20261113-berdKH/cold-proof-70db0868-primary-sources-gpt6sol'
PARENT='/home/brittonr/sbfp-20261113-berdKH/native-adoption-70db0868-primary-sources-gpt6sol'
ADOPTER='/home/brittonr/sbfp-20261113-berdKH/bin/adopt-source-built-stagex-70db0868-primary-sources'
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
SIGNING_KEY='/home/brittonr/sbfp-20261113-berdKH/native-row-qualification-20261115.axozfy/config/signing-key'
test "$(cat "$COLD/driver.exit")" = 0
jq -e '.verified == true and .status == "verified"' "$COLD/trust-report.json"
test "$("$B3" --no-names "$ADOPTER")" = fa0be4a457b864ae7ad545fab1c0304a2eecb3c733a8b549ae1c94faab6a2873
test ! -e "$PARENT"
test ! -L "$PARENT"
mkdir "$PARENT"
exec > "$PARENT/execution.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "stagex_adoption_exit=%s\n" "$status"; printf "%s\n" "$status" > "$PARENT/execution.exit"; exit "$status"' EXIT
set -x
mkdir "$PARENT/home" "$PARENT/tmp"
env -i HOME="$PARENT/home" TMPDIR="$PARENT/tmp" PATH='/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin:/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin:/run/current-system/sw/bin:/usr/bin:/bin' CRUNCH_NO_FUSE=1 "$ADOPTER" --json --proof-root "$COLD/proof" --expected-plan-blake3 "$PLAN_BLAKE3" --expected-receipt-bytes-blake3 "$RECEIPT_BYTES_BLAKE3" --store "$PARENT/store" --state-dir "$PARENT/state" --signing-key "$SIGNING_KEY" --report "$PARENT/adoption-report.json" --backend snix > "$PARENT/command-report.json"
"$B3" "$PARENT/adoption-report.json"
finished=1
printf 'stagex_adoption_complete=1\n'
