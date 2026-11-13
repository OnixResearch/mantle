#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
exec > "$RUN/prepared-cohort-verification.log" 2>&1
trap 's=$?; printf "prepared_cohort_verification_exit=%s\n" "$s"; printf "%s\n" "$s" > "$RUN/prepared-cohort-verification.exit"; exit "$s"' EXIT
set -x
date -Is
COHORT="$RUN/prepared-cohort-72f6d21f"
test "$("$B3" --no-names "$COHORT/source-profile.json")" = 0b63f4bcacadf36f904559c63793836d2cf17003aad3faeec66f940573139372
test "$("$B3" --no-names "$COHORT/source-program-preparation.json")" = ccb08afa9a223386dd3a8f8853068fb39f75c26f2ad43e2d9b3736b95ac1398a
"$B3" --check "$RUN/frozen-runner-blake3.txt"
env -i HOME="$RUN/preparation-72f6d21f-home" TMPDIR="$RUN/preparation-72f6d21f-tmp" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$RUN/bin/mantle-frozen" --json source bundle verify --from "$COHORT/source-profile.json" > "$RUN/prepared-source-profile-verification.json"
jq -e '.manifest_blake3 == "47c446b61809bc1f968a065969431af0bb3d7791227e8d18e0bacf95bcad6d77" and .ready_class == "ready" and .missing_records == [] and .stale_records == [] and .unsupported_records == [] and .untrusted_records == []' "$RUN/prepared-source-profile-verification.json"
date -Is
