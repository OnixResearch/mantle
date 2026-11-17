#!/usr/bin/env bash
set -euo pipefail
cd '/home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources'
test "$#" -eq 1
ADOPTION_BYTES_BLAKE3="$1"
[[ "$ADOPTION_BYTES_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
ADOPTION='/home/brittonr/sbfp-20261113-berdKH/native-adoption-70db0868-primary-sources-gpt6sol'
SOURCE='/home/brittonr/sbfp-20261113-berdKH/runner-source-70db0868-primary-sources'
COHORT='/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-70db0868-primary-sources'
RUN='/home/brittonr/sbfp-20261113-berdKH/native-rows-70db0868-primary-sources-gpt6sol'
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
LOG='/home/brittonr/sbfp-20261113-berdKH/native-rows-70db0868-primary-sources-gpt6sol.execution.log'
EXIT_FILE='/home/brittonr/sbfp-20261113-berdKH/native-rows-70db0868-primary-sources-gpt6sol.execution.exit'
test "$(cat "$ADOPTION/execution.exit")" = 0
test "$("$B3" --no-names "$ADOPTION/adoption-report.json")" = "$ADOPTION_BYTES_BLAKE3"
test ! -e "$RUN"
test ! -L "$RUN"
test ! -e "$LOG"
exec > "$LOG" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "native_rows_exit=%s\n" "$status"; printf "%s\n" "$status" > "$EXIT_FILE"; exit "$status"' EXIT
set -x
env -i HOME="$ADOPTION/home" TMPDIR="$ADOPTION/tmp" PATH='/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin:/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin:/run/current-system/sw/bin:/usr/bin:/bin' CRUNCH_NO_FUSE=1 '/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' "$SOURCE/scripts/qualify-native-rows.py" --mantle '/home/brittonr/sbfp-20261113-berdKH/bin/mantle-70db0868-primary-sources' --checker '/home/brittonr/sbfp-20261113-berdKH/bin/check-native-row-receipts-70db0868-primary-sources' --tar '/nix/store/80c50gsgpxymdhvs5c2vvnq4zfc44k2r-gnutar-1.35/bin/tar' --b3sum "$B3" --source-root "$SOURCE" --source-profile "$COHORT/source-profile.json" --adoption-report "$ADOPTION/adoption-report.json" --signing-key '/home/brittonr/sbfp-20261113-berdKH/native-row-qualification-20261115.axozfy/config/signing-key' --run-root "$RUN" --store-backend snix --jobs 16 --build-seconds 86400 --expected-manifest-blake3 28fa8ae91ce978216fba288452219bc0c5aa07a7b57fc6f92a7168863b1b1aab --expected-adoption-blake3 "$ADOPTION_BYTES_BLAKE3" --expected-mantle-blake3 a535bb5fe3fda338469a4139f72c3ff5b8a59c07061b0a2ab33197ee38337019 --expected-checker-blake3 817a3c70959947d69c2bf58a114bbc444770a4545ffff677794b137bae7ca879 --expected-source-tree-blake3 0e5731f87c8cfe87c9dfa9d75ee8d43f3684f86b76479aa28dfe5b27908721b1
finished=1
printf 'native_rows_complete=1\n'
