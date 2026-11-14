#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
SOURCE="$RUN/runner-source-22eb45ca"
MANTLE="$RUN/bin/mantle-22eb45ca"
COHORT="$RUN/prepared-cohort-22eb45ca-resumed"
BWRAP=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap
SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
exec > "$RUN/preparation-resumed-22eb45ca.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "preparation_resumed_22eb45ca_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/preparation-resumed-22eb45ca.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
hostname
uname -srm
cat /proc/sys/kernel/random/boot_id
df -B1 "$RUN"
free -b
test "$(cat "$RUN/qualification-22eb45ca.exit")" = 0
test "$(cat "$RUN/canonical-qualification-22eb45ca.exit")" = 0
test "$("$B3" --no-names "$MANTLE")" = ebef9e67e8498b20ba843c6a8155b3156c89aa015dd240bfb17789b612bd8518
"$MANTLE" --version
test -x "$BWRAP"
test -x "$SANDBOX_SHELL"
"$B3" "$BWRAP" "$SANDBOX_SHELL"
"$BWRAP" --unshare-all --unshare-user --ro-bind / / -- "$SANDBOX_SHELL" true
cd "$SOURCE"
SOURCE_DIGEST=$(tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names)
test "$SOURCE_DIGEST" = e090c7f6af61a943f047257c2c48df3abcb039828d0d4b56b384116829b28edf
PROFILE_ID=570d65093f49efbcbd4cb73af3b0bbb0783788c101d57cdc352b44e55eea0ebd
jq --arg profile "$PROFILE_ID" -e '.input_manifest_blake3 == "de163b8deb0b14afd58469f0c66f8dc62714117b4706323d5e419cf37cd03c6c" and .output_manifest_blake3 == $profile and .replacement_mantle_source_blake3 == "aa317a2b73be17013f08d35775ea99f0828b6f345b924d1ce6882a2d51b9b978" and .preserved_record_count == 65 and .added_record_count == 0' "$RUN/source-refresh-22eb45ca.json"
PROFILE_BYTES_BEFORE=$("$B3" --no-names "$RUN/source-profile-22eb45ca.json")
PROOF_HOME="$RUN/preparation-resumed-22eb45ca-home"
PROOF_TMP="$RUN/preparation-resumed-22eb45ca-tmp"
mkdir "$PROOF_HOME" "$PROOF_TMP"
test ! -e "$COHORT"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$RUN/source-profile-22eb45ca.json" > "$RUN/source-profile-verification-resumed-22eb45ca.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/source-profile-verification-resumed-22eb45ca.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --prepare-source-programs --source-profile "$RUN/source-profile-22eb45ca.json" --expected-source-profile-blake3 "$PROFILE_ID" --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap "$BWRAP" --proof-sandbox-shell "$SANDBOX_SHELL" --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$COHORT" > "$RUN/preparation-report-resumed-22eb45ca.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$COHORT/source-profile.json" > "$RUN/prepared-source-profile-verification-resumed-22eb45ca.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/prepared-source-profile-verification-resumed-22eb45ca.json"
PROFILE_BYTES_AFTER=$("$B3" --no-names "$RUN/source-profile-22eb45ca.json")
test "$PROFILE_BYTES_BEFORE" = "$PROFILE_BYTES_AFTER"
"$B3" "$RUN/source-profile-22eb45ca.json" "$COHORT/source-profile.json" "$COHORT/source-program-preparation.json"
date -Is
finished=1
printf 'preparation_resumed_complete=1\n'
