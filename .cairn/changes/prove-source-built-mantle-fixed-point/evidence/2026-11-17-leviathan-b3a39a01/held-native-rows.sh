#!/usr/bin/env bash
set -euo pipefail
cd '/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source'
test "$#" -eq 1
ADOPTION_BYTES_BLAKE3="$1"
[[ "$ADOPTION_BYTES_BLAKE3" =~ ^[a-f0-9]{64}$ ]]
ADOPTION='/home/brittonr/sbfp-20261113-berdKH/native-adoption-b3a39a01-immutable-source-gpt6sol'
SOURCE='/home/brittonr/sbfp-20261113-berdKH/runner-source-b3a39a01-immutable-source'
COHORT='/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-b3a39a01-immutable-source'
RUN='/home/brittonr/sbfp-20261113-berdKH/native-rows-b3a39a01-immutable-source-gpt6sol'
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
LOG='/home/brittonr/sbfp-20261113-berdKH/native-rows-b3a39a01-immutable-source-gpt6sol.execution.log'
EXIT_FILE='/home/brittonr/sbfp-20261113-berdKH/native-rows-b3a39a01-immutable-source-gpt6sol.execution.exit'
test "$(cat "$ADOPTION/execution.exit")" = 0
test "$("$B3" --no-names "$ADOPTION/adoption-report.json")" = "$ADOPTION_BYTES_BLAKE3"
test ! -e "$RUN"
test ! -L "$RUN"
test ! -e "$LOG"
exec > "$LOG" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "native_rows_exit=%s\n" "$status"; printf "%s\n" "$status" > "$EXIT_FILE"; exit "$status"' EXIT
set -x
env -i HOME="$ADOPTION/home" TMPDIR="$ADOPTION/tmp" PATH='/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin:/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin:/run/current-system/sw/bin:/usr/bin:/bin' CRUNCH_NO_FUSE=1 '/nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14' "$SOURCE/scripts/qualify-native-rows.py" --mantle '/home/brittonr/sbfp-20261113-berdKH/bin/mantle-b3a39a01-immutable-source' --checker '/home/brittonr/sbfp-20261113-berdKH/bin/check-native-row-receipts-b3a39a01-immutable-source' --tar '/nix/store/80c50gsgpxymdhvs5c2vvnq4zfc44k2r-gnutar-1.35/bin/tar' --b3sum "$B3" --source-root "$SOURCE" --source-profile "$COHORT/source-profile.json" --adoption-report "$ADOPTION/adoption-report.json" --signing-key '/home/brittonr/sbfp-20261113-berdKH/native-row-qualification-20261115.axozfy/config/signing-key' --run-root "$RUN" --store-backend snix --jobs 16 --build-seconds 86400 --expected-manifest-blake3 8c86d017b2cc676d216f2bdbdc7b39448fa200aced5c5ef7eca98af4402cb4c4 --expected-adoption-blake3 "$ADOPTION_BYTES_BLAKE3" --expected-mantle-blake3 9c13e989f2410ecca83469ff1aec7faade0672186bdbf1759a68604cd0c3ab84 --expected-checker-blake3 17e18d4c5a17ec709da5a734233e03a9eac123853561a02d1d141ac2863a344e --expected-source-tree-blake3 45344056501e2ed0c23b1a122d0dd300b02cf0fc0d2fe5e0808d6159a93e60be
finished=1
printf 'native_rows_complete=1\n'
