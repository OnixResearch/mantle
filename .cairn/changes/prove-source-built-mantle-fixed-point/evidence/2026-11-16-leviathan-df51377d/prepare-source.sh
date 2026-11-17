#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
NAME=df51377d-unit-plans
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
SOURCE=/home/brittonr/sbfp-20261113-berdKH/runner-source-df51377d-unit-plans
MANTLE="$RUN/bin/mantle-$NAME"
COHORT="$RUN/prepared-cohort-$NAME"
BWRAP=/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap
SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
test ! -e "$RUN/preparation-$NAME.log"
exec > "$RUN/preparation-$NAME.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ]; then if [ "$finished" -ne 1 ]; then status=1; fi; fi; printf "preparation_exit=%s\n" "$status"; printf "%s\n" "$status" > "$RUN/preparation-$NAME.exit"; exit "$status"' EXIT
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
test "$(cat "$RUN/qualification-$NAME.exit")" = 0
test "$(cat "$RUN/canonical-qualification-$NAME.exit")" = 0
"$B3" --check "$RUN/runner-blake3-$NAME.txt"
"$MANTLE" --version
test "$("$B3" --no-names "$BWRAP")" = 9979941d82a18db70c4fa1bed4cd3d33504c1d8b35b9f08b6784e494abbdcece
test "$("$B3" --no-names "$SANDBOX_SHELL")" = ac277a3b87ae477a75067b89de22865222d1d01e1cb7f91ca5f5eeca8e8ef6bc
"$BWRAP" --unshare-all --unshare-user --ro-bind / / -- "$SANDBOX_SHELL" true
cd "$SOURCE"
SOURCE_BEFORE=$(tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names)
test "$SOURCE_BEFORE" = "$(jq -er .complete_archive_blake3 "/home/brittonr/sbfp-20261113-berdKH/complete-source-df51377d-unit-plans.json")"
PROOF_HOME="$RUN/preparation-$NAME-home"
PROOF_TMP="$RUN/preparation-$NAME-tmp"
mkdir "$PROOF_HOME" "$PROOF_TMP"
test ! -e "$COHORT"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle refresh-mantle-source --from "$RUN/incoming/source-profile-baseline.json" --mantle-source "$SOURCE" --to "$RUN/source-profile-$NAME.json" > "$RUN/source-refresh-$NAME.json"
jq -e '.input_manifest_blake3 == "de163b8deb0b14afd58469f0c66f8dc62714117b4706323d5e419cf37cd03c6c" and .previous_mantle_source_blake3 == "51b3da0e5d3a15f690aec40a41763ff7b656eee03b2e399d97b5d705e6005194" and .preserved_record_count == 65 and .added_record_count == 0' "$RUN/source-refresh-$NAME.json"
PROFILE_ID=$(jq -er .output_manifest_blake3 "$RUN/source-refresh-$NAME.json")
[[ "$PROFILE_ID" =~ ^[a-f0-9]{64}$ ]]
PROFILE_BYTES_BEFORE=$("$B3" --no-names "$RUN/source-profile-$NAME.json")
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$RUN/source-profile-$NAME.json" > "$RUN/source-profile-verification-$NAME.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/source-profile-verification-$NAME.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json self-build --source-built-fixed-point --prepare-source-programs --source-profile "$RUN/source-profile-$NAME.json" --expected-source-profile-blake3 "$PROFILE_ID" --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f --proof-bwrap "$BWRAP" --proof-sandbox-shell "$SANDBOX_SHELL" --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 --proof-exec-events-max 524288 --proof-source-records-max 65536 --jobs 16 --out "$COHORT" > "$RUN/preparation-report-$NAME.json"
env -i HOME="$PROOF_HOME" TMPDIR="$PROOF_TMP" PATH=/usr/bin:/bin CRUNCH_NO_FUSE=1 "$MANTLE" --json source bundle verify --from "$COHORT/source-profile.json" > "$RUN/prepared-source-profile-verification-$NAME.json"
jq -e '.ready_class == "ready" and ([.missing_records, .stale_records, .unsupported_records, .untrusted_records] | all(length == 0))' "$RUN/prepared-source-profile-verification-$NAME.json"
test "$PROFILE_BYTES_BEFORE" = "$("$B3" --no-names "$RUN/source-profile-$NAME.json")"
SOURCE_AFTER=$(tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names)
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
"$B3" "$RUN/source-profile-$NAME.json" "$COHORT/source-profile.json" "$COHORT/source-program-preparation.json"
date -Is
finished=1
printf 'preparation_complete=1\n'
